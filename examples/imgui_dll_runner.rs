mod runner_modules;

use hackers::metadata::HaCKLoadType;
use runner_modules::{event_handler, ImguiContext, RunnerHost, WgpuRenderer};
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

use hackers::{impl_hac_boilerplate, HaCK, HaCKS, HaCMetadata};
use serde::Serialize;
use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize)]
struct DummyModule {
    cfg: bool,
    metadata: HaCMetadata,
}

impl HaCK for DummyModule {
    fn name(&self) -> &str {
        "Dummy Internal Module"
    }

    fn nac_type_id(&self) -> TypeId {
        TypeId::of::<DummyModule>()
    }

    fn update(&mut self, _hacs: &HaCKS) {}

    fn render_window(&mut self, ui: &dyn hackers::UiBackend) {
        self.render_menu(ui);
    }
    fn render_menu(&mut self, ui: &dyn hackers::UiBackend) {
        ui.text("Hello from Dummy Internal Module!");
        if ui.button("Click Me") {
            println!("Dummy module button clicked!");
        }
    }

    impl_hac_boilerplate!(DummyModule, metadata);
}

async fn run(event_loop: EventLoop<()>, window: Arc<winit::window::Window>) {
    let mut imgui_ctx = ImguiContext::new(&window);
    let mut wgpu_renderer = WgpuRenderer::new(window.clone(), imgui_ctx.context_mut()).await;

    // Create RunnerHost with internal modules
    let mut runner_host = RunnerHost::new().with_internal_modules(|| {
        vec![Rc::new(RefCell::new(DummyModule {
            cfg: true,
            metadata: HaCMetadata {
                is_window_enabled: false,
                load_type: HaCKLoadType::Internal,
                ..HaCMetadata::default()
            },
        }))]
    });

    // Trigger initial background load if configured
    runner_host.queued_bg_image = Some(runner_host.bg_image_path_input.clone());

    let _ = event_loop.run(move |event, elwt| {
        event_handler::handle_event(
            event,
            elwt,
            &window,
            &mut wgpu_renderer,
            &mut imgui_ctx,
            &mut runner_host,
        );
    });
}
