use std::sync::Arc;

use winit::event::WindowEvent;
use winit::window::Window;

use anyhow::{Result, Context};

pub struct State<'a> {
    window:   Arc<Window>,
    surface:  wgpu::Surface<'a>,
    device:   wgpu::Device,
    queue:    wgpu::Queue,
    config:   wgpu::SurfaceConfiguration,
    size:     winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
}

impl<'a> State<'a> {
    pub async fn new(window: Window) -> Result<State<'a>> {
        let window = Arc::new(window);

        let size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        let size = winit::dpi::PhysicalSize {
            width:  size.width / 2,
            height: size.height / 2,
        };

        let backend = wgpu::Backends::PRIMARY;

        #[cfg(target_arch = "wasm32")]
        let backend = wgpu::Backends::GL;

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: backend,
                ..Default::default()
            },
        );

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.context("")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ).await.unwrap();

        let caps = surface.get_capabilities(&adapter);

        let format =
            caps.formats
                .iter()
                .copied()
                .find(|format| format.is_srgb())
                .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

            width:  size.width,
            height: size.height,

            present_mode: caps.present_modes[0],
            alpha_mode:   caps.alpha_modes[0],

            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(
            wgpu::include_wgsl!("shaders/shader.wgsl"),
        );

        let pipeline = Self::create_pipeline(
            &shader, &device, &config, &[],
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            window,
        })
    }

    fn create_pipeline(
        shader:  &wgpu::ShaderModule,
        device:  &wgpu::Device,
        config:  &wgpu::SurfaceConfiguration,
        layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline_layout"),
                bind_group_layouts:   layouts,
                push_constant_ranges: &[],
            },
        );

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label:  Some("pipeline"),
                layout: Some(&pipeline_layout),

                vertex: wgpu::VertexState {
                    module:      shader,
                    entry_point: "vs_main",
                    buffers:     &[],
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                },

                fragment: Some(
                    wgpu::FragmentState {
                        module:      shader,
                        entry_point: "fs_main",

                        targets: &[Some(wgpu::ColorTargetState {
                            format:     config.format,
                            blend:      Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],

                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
                    },
                ),

                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask:  !0,
                    alpha_to_coverage_enabled: false,
                },

                multiview: None,
                cache: None,
            }
        )
    }

    pub fn scale(&mut self, factor: &f64) {
        self.resize(
            winit::dpi::PhysicalSize::new(
                (self.size.width  as f64 * factor) as u32,
                (self.size.height as f64 * factor) as u32,
            )
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        let new_size = winit::dpi::PhysicalSize {
            width:  new_size.width  / 2,
            height: new_size.height / 2,
        };

        self.size = new_size;

        self.config.width  = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
    }

    pub fn event(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self, dt: instant::Duration) {
    }

    pub fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default(),
        );

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            }
        );

        {
            let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),

                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,

                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.1,
                                        g: 0.4,
                                        b: 0.7,
                                        a: 1.0,
                                    },
                                ),
                                store: wgpu::StoreOp::Store,
                            },
                        }
                    )],

                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                },
            );

            rpass.set_pipeline(&self.pipeline);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
