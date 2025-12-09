use std::collections::HashMap;
use std::sync::Arc;
use winit::window::Window;

pub struct WgpuRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub renderer: imgui_wgpu::Renderer,
    /// Maps plugin texture request IDs to actual GPU TextureIds
    pub texture_id_map: HashMap<u64, imgui::TextureId>,
    pub clear_color: wgpu::Color,
}

impl WgpuRenderer {
    pub async fn new(window: Arc<Window>, imgui: &mut imgui::Context) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: config.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(imgui, &device, &queue, renderer_config);

        Self {
            device,
            queue,
            surface,
            config,
            renderer,
            texture_id_map: HashMap::new(),
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, draw_data: &imgui::DrawData) -> Result<(), wgpu::SurfaceError> {
        // FIRST: Process any queued texture uploads from plugins
        use hackers::impl_backends::imgui_backend::take_texture_upload_requests;
        let uploads = take_texture_upload_requests();

        for (request_id, width, height, data) in uploads {
            let texture_config = imgui_wgpu::TextureConfig {
                label: Some("imgui_uploaded_texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                ..Default::default()
            };

            let texture = imgui_wgpu::Texture::new(&self.device, &self.renderer, texture_config);
            texture.write(&self.queue, &data, width, height);

            let texture_id = self.renderer.textures.insert(texture);

            // Store mapping so plugins can look up the GPU texture ID
            self.texture_id_map.insert(request_id, texture_id);
        }

        use hackers::impl_backends::imgui_backend::set_texture_id_map;
        set_texture_id_map(self.texture_id_map.clone());

        let output_frame = self.surface.get_current_texture()?;
        let view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer
                .render(draw_data, &self.queue, &self.device, &mut rpass)
                .expect("Rendering failed");
        }

        self.queue.submit(Some(encoder.finish()));
        output_frame.present();

        Ok(())
    }

    pub fn set_clear_color(&mut self, color: [f64; 4]) {
        self.clear_color = wgpu::Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        };
    }

    pub fn create_texture(
        &mut self,
        image: &image::DynamicImage,
        label: Option<&str>,
    ) -> imgui::TextureId {
        let (width, height) = (image.width(), image.height());
        let rgba = image.to_rgba8();
        let data = rgba.as_raw();

        let texture_config = imgui_wgpu::TextureConfig {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&self.device, &self.renderer, texture_config);
        texture.write(&self.queue, data, width, height);

        self.renderer.textures.insert(texture)
    }
}
