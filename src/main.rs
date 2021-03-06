mod vertex;
mod camera;
mod shapes;
mod primitive_renderer;
use vertex::Vertex;

use primitive_renderer::PrimitiveRenderer;

use iced_wgpu::{wgpu, Backend, Renderer, Settings};
use iced_winit::{futures, winit};

use futures::task::SpawnExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub fn main() {
    // Initialize winit
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    let use_dx12 = std::env::args().skip(1).any(|s| s == "dx12");
    let use_iced = std::env::args().skip(1).any(|s| s == "iced");

    let backend = if use_dx12 {
        wgpu::BackendBit::DX12
    } else {
        wgpu::BackendBit::DX11
    };

    // Initialize wgpu
    let instance = wgpu::Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window) };

    let (mut device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Request adapter");

        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: false,
                },
                None,
            )
            .await
            .expect("Request device")
    });

    let mut swap_chain = {
        let size = window.inner_size();
        device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: SWAPCHAIN_FORMAT,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        )
    };
    let mut swapchain_rebuild = false;

    // Initialize staging belt and local pool
    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
    let mut staging_belt_pool = futures::executor::LocalPool::new();

    // Initialize primitive rendering
    let mut primitive_renderer = PrimitiveRenderer::new(&mut device, SWAPCHAIN_FORMAT);
    let camera = camera::Camera::default();
    let grid = shapes::grid(30, 1.);
    primitive_renderer.set_lines(&device, &grid);

    // Initialize iced
    if use_iced {
        let _renderer = Renderer::new(Backend::new(&mut device, Settings::default()));
    }

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::Resized(_) = &event {
                    swapchain_rebuild = true; // Rebuild swapchain
                }
            }
            Event::MainEventsCleared => {
                // Rebuild the swapchain if necessary
                let size = window.inner_size();
                if swapchain_rebuild {
                    swap_chain = device.create_swap_chain(
                        &surface,
                        &wgpu::SwapChainDescriptor {
                            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                            format: SWAPCHAIN_FORMAT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::Mailbox,
                        },
                    );
                }

                // Get viewports from the partition
                swapchain_rebuild = false;

                // Begin rendering another frame
                let frame = swap_chain.get_current_frame().expect("Next frame");

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // Draw the primitive renderer
                primitive_renderer.draw(&mut encoder, &frame.output.view, size);

                // Update camera matrices
                let matrix = camera.matrix(size.width, size.height);
                primitive_renderer.set_camera_matrix(&queue, matrix.as_slice());

                // Then we submit the work
                staging_belt.finish();
                queue.submit(Some(encoder.finish()));

                // And recall staging buffers
                staging_belt_pool
                    .spawner()
                    .spawn(staging_belt.recall())
                    .expect("Recall staging buffers");

                staging_belt_pool.run_until_stalled();
            }
            _ => {}
        }
    })
}