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

            // Render frame
            let draw_data = imgui_ctx.context_mut().render();
            if let Err(wgpu::SurfaceError::Outdated) = wgpu_renderer.render(draw_data) {
                return;
            }
        }
        _ => (),
    }
}
