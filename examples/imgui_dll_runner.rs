mod runner_modules;

use runner_modules::{event_handler, AppState, ImguiContext, WgpuRenderer};
use std::sync::Arc;
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
    let mut imgui_ctx = ImguiContext::new(&window);
    let mut wgpu_renderer = WgpuRenderer::new(window.clone(), imgui_ctx.context_mut()).await;
    let mut app_state = AppState::new();

    let _ = event_loop.run(move |event, elwt| {
        event_handler::handle_event(
            event,
            elwt,
            &window,
            &mut wgpu_renderer,
            &mut imgui_ctx,
            &mut app_state,
        );
    });
}
