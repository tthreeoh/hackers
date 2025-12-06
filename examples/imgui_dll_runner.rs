use hackers::gui::UiBackend;
use hackers::hackrs::HaCKS::HaCKS;
use hackers::impl_backends::imgui_backend::ImguiBackend;
use imgui::{Context, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Hackers Imgui WGPU Runner (DX12)")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .build(&event_loop)
            .unwrap(),
    );

    pollster::block_on(run(event_loop, window));
}

async fn run(event_loop: EventLoop<()>, window: Arc<winit::window::Window>) {
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

    let mut config = wgpu::SurfaceConfiguration {
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

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui
        .fonts()
        .add_font(&[FontSource::DefaultFontData { config: None }]);

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

    let renderer_config = imgui_wgpu::RendererConfig {
        texture_format: config.format,
        ..Default::default()
    };

    let mut renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, renderer_config);

    let mut last_frame = Instant::now();

    let mut hacks = HaCKS::new();
    let mut dll_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dll_path.push("examples");
    dll_path.push("example_dll");
    dll_path.push("target");
    dll_path.push("debug");
    dll_path.push("example_dll.dll");

    if dll_path.exists() {
        println!("Loading DLL: {:?}", dll_path);
        hacks.load_dynamic(&dll_path).expect("Failed to load DLL");
    }

    let _ = event_loop.run(move |event, elwt| {
        platform.handle_event(imgui.io_mut(), &window, &event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(&device, &config);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => return,
                    Err(e) => {
                        eprintln!("Surface error: {:?}", e);
                        return;
                    }
                };
                let view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.new_frame();

                if let Some(_menu_bar) = ui.begin_main_menu_bar() {
                    let backend = ImguiBackend::new(ui);
                    hacks.render_menu(&backend);
                }

                {
                    let backend = ImguiBackend::new(ui);

                    ui.window("Hackers Host Window")
                        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.text("Running on WGPU (DX12/Vulkan)");
                            ui.separator();
                            let io = ui.io();
                            ui.text(format!("FPS: {:.1}", io.framerate));
                        });

                    hacks.render_window(&backend);
                }

                ui.show_demo_window(&mut true);

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.1,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    renderer
                        .render(imgui.render(), &queue, &device, &mut rpass)
                        .expect("Rendering failed");
                }

                queue.submit(Some(encoder.finish()));
                output_frame.present();
            }
            _ => (),
        }
    });
}
