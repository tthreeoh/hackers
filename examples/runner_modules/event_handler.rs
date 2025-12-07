use crate::runner_modules::{host_ui, plugin_ui, ImguiContext, WgpuRenderer};
use hackers::hackrs::HaCKS::HaCKS;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

pub struct AppState {
    pub hacks: HaCKS,
    pub unload_queue: Vec<usize>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            hacks: HaCKS::new(),
            unload_queue: Vec::new(),
        }
    }

    pub fn process_unload_queue(&mut self) {
        if !self.unload_queue.is_empty() {
            // Sort in reverse order to preserve indices when removing
            self.unload_queue.sort_by(|a, b| b.cmp(a));
            for idx in self.unload_queue.drain(..) {
                println!("Unloading plugin at index {}", idx);
                if let Err(e) = self.hacks.unload_dynamic(idx) {
                    eprintln!("Failed to unload plugin: {}", e);
                }
            }
        }
    }
}

pub fn handle_event(
    event: Event<()>,
    elwt: &EventLoopWindowTarget<()>,
    window: &Arc<Window>,
    wgpu_renderer: &mut WgpuRenderer,
    imgui_ctx: &mut ImguiContext,
    app_state: &mut AppState,
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
            let ui = imgui_ctx.prepare_frame(window);

            // Render UI components
            host_ui::render_menu_bar(ui, &mut app_state.hacks);
            host_ui::render_host_window(ui, &mut app_state.hacks);
            plugin_ui::render_plugin_manager(ui, &mut app_state.hacks, &mut app_state.unload_queue);

            ui.show_demo_window(&mut true);

            // Process unload queue after UI rendering
            app_state.process_unload_queue();

            // Render frame
            let draw_data = imgui_ctx.context_mut().render();
            if let Err(wgpu::SurfaceError::Outdated) = wgpu_renderer.render(draw_data) {
                return;
            }
        }
        _ => (),
    }
}
