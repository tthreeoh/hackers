use imgui::{Context, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use winit::event::Event;
use winit::window::Window;

pub struct ImguiContext {
    context: Context,
    platform: WinitPlatform,
    last_frame: Instant,
}

impl ImguiContext {
    pub fn new(window: &Window) -> Self {
        let mut context = Context::create();
        context.set_ini_filename(None);
        context
            .fonts()
            .add_font(&[FontSource::DefaultFontData { config: None }]);

        let mut platform = WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);

        Self {
            context,
            platform,
            last_frame: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }

    pub fn prepare_frame(&mut self, window: &Window) -> &mut imgui::Ui {
        let now = Instant::now();
        self.context
            .io_mut()
            .update_delta_time(now - self.last_frame);
        self.last_frame = now;

        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");

        self.context.new_frame()
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}
