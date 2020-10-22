use crate::vertex::Vertex;
use iced_winit::winit::dpi::PhysicalSize;
use iced_wgpu::wgpu;
use wgpu::util::DeviceExt;

pub struct PrimitiveRenderer {
    lines: Option<(wgpu::Buffer, usize)>,
    lines_pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl PrimitiveRenderer {
    pub fn new(device: &wgpu::Device, swapchain_format: wgpu::TextureFormat) -> PrimitiveRenderer {
        // Load the shaders from disk
        let vs_module = device.create_shader_module(wgpu::include_spirv!("shaders/unlit.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shaders/unlit.frag.spv"));

        // Uniform buffer
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<[f32; 16]>() as _,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layout (basically a descriptorset layout)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
            }],
            label: None,
        });

        // Vertex descriptor
        let vertex_state = wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<Vertex>() as _,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 3 * std::mem::size_of::<f32>() as u64,
                        shader_location: 1,
                    },
                ],
            }],
        };

        let mut pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::LineList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: swapchain_format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: vertex_state.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let lines_pipeline = device.create_render_pipeline(&pipeline_desc);

        Self {
            lines: None,
            lines_pipeline,
            uniform_buf,
            bind_group,
        }
    }

    /// Issue draw commands to `encoder` for `target` within `area`
    pub fn draw<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        target: &'a wgpu::TextureView,
        area: PhysicalSize<u32>,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_viewport(
            0.,
            0.,
            area.width as f32,
            area.height as f32,
            0.0,
            1.0,
        );
        rpass.set_scissor_rect(
            0,
            0,
            area.width,
            area.height,
        );
        rpass.set_bind_group(0, &self.bind_group, &[]);

        if let Some((lines, n_vertices)) = &self.lines {
            rpass.set_pipeline(&self.lines_pipeline);
            rpass.set_vertex_buffer(0, lines.slice(..));
            rpass.draw(0..*n_vertices as u32, 0..1);
        }
    }

    /// Set camera matrix
    pub fn set_camera_matrix(&self, queue: &wgpu::Queue, data: &[f32]) {
        assert_eq!(data.len(), 16);
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(data));
    }

    /// Set line vertices and colors
    pub fn set_lines(&mut self, device: &wgpu::Device, lines: &[Vertex]) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Buffer"),
            contents: bytemuck::cast_slice(&lines),
            usage: wgpu::BufferUsage::VERTEX,
        });
        self.lines = Some((buffer, lines.len()));
    }
}
