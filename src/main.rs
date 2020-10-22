mod vertex;
mod camera;
mod screen_multiplexer;
mod shapes;
mod primitive_renderer;
use futures::FutureExt;
use futures::StreamExt;
use std::fs;
use vertex::Vertex;

use primitive_renderer::PrimitiveRenderer;

use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{conversion, futures, program, winit, Debug, Size};

use futures::task::SpawnExt;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn convert_size(size: PhysicalSize<u32>) -> Size<f32> {
    Size::new(size.width as f32, size.height as f32)
}

fn convert_size_u32(size: PhysicalSize<u32>) -> Size<u32> {
    Size::new(size.width, size.height)
}

const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub fn main() {
    // Initialize winit
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
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
    let mut camera = camera::Camera::default();
    let grid = shapes::grid(30, 1.);
    primitive_renderer.set_lines(&device, &grid);

    // Initialize iced
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&mut device, Settings::default()));

    let mut multiplexer = screen_multiplexer::ScreenMultiplexer::new(500, window.inner_size());
    let (left_area, _) = multiplexer.areas();

    // Set up command pool
    let thread_pool = futures::executor::ThreadPool::new().unwrap();

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

                if let Some(event) = event.to_static() {
                    let (left, right) = multiplexer.event(event);
                    let (_, cursor_position) = multiplexer.cursors();
                }
            }
            Event::MainEventsCleared => {
                // Rebuild the swapchain if necessary
                if swapchain_rebuild {
                    let size = window.inner_size();
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
                let (left_area, right_area) = multiplexer.areas();

                swapchain_rebuild = false;

                // Begin rendering another frame
                let frame = swap_chain.get_current_frame().expect("Next frame");

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // Draw the primitive renderer
                primitive_renderer.draw(&mut encoder, &frame.output.view, right_area);

                // Update camera matrices
                let matrix = camera.matrix(right_area.size.width, right_area.size.height);
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