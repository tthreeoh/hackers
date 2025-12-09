use crate::runner_modules::{ImguiContext, RunnerHost, WgpuRenderer};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

// AppState is now replaced by RunnerHost in runner_host.rs

pub fn handle_event(
    event: Event<()>,
    elwt: &EventLoopWindowTarget<()>,
    window: &Arc<Window>,
    wgpu_renderer: &mut WgpuRenderer,
    imgui_ctx: &mut ImguiContext,
    runner_host: &mut RunnerHost,
) {
    imgui_ctx.handle_event(window, &event);

    match event {
        Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } => {
            wgpu_renderer.resize(new_size.width, new_size.height);
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
            // Scope for UI rendering - ui must be dropped before calling render()
            {
                let ui = imgui_ctx.prepare_frame(window);
                runner_host.render_ui(ui);
            } // ui is dropped here

            // Update Clear Color from RunnerHost
            let color_f64 = [
                runner_host.clear_color[0] as f64,
                runner_host.clear_color[1] as f64,
                runner_host.clear_color[2] as f64,
                runner_host.clear_color[3] as f64,
            ];
            wgpu_renderer.set_clear_color(color_f64);

            // Handle Queued Background Image Load
            if let Some(path) = runner_host.queued_bg_image.take() {
                match image::open(&path) {
                    Ok(img) => {
                        let width = img.width();
                        let height = img.height();
                        let texture_id =
                            wgpu_renderer.create_texture(&img, Some("background_image"));
                        runner_host.background_image = Some(texture_id);
                        runner_host.bg_image_size = [width, height];
                        println!("Loaded background image: {} ({}x{})", path, width, height);
                    }
                    Err(e) => {
                        eprintln!("Failed to load background image '{}': {}", path, e);
                    }
                }
            }

            // Render frame
            let draw_data = imgui_ctx.context_mut().render();
            if let Err(wgpu::SurfaceError::Outdated) = wgpu_renderer.render(draw_data) {
                return;
            }
        }
        _ => (),
    }
}
