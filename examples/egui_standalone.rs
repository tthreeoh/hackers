use eframe::egui;
use hackers::gui::{TreeNodeFlags, UiBackend};
use hackers::hackrs::HaCKS::HaCKS;
use hackers::impl_backends::egui_backend::EguiBackend;
use hackers::{impl_hac_boilerplate, HaCK, HaCMetadata};
use serde::{Deserialize, Serialize};
use std::borrow::Cow::Borrowed;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct TestPlugin {
    show_window: bool,
    show_menu: bool,
    hac_data: HaCMetadata,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            show_window: true,
            show_menu: true,
            hac_data: HaCMetadata {
                name: Borrowed("Test Plugin"),
                description: Borrowed("Test Plugin"),
                category: Borrowed("Test"),
                visible_in_gui: true,
                is_update_enabled: true,
                is_render_enabled: true,
                window_size: [106.0, 108.0],
                ..HaCMetadata::default()
            },
        }
    }
}

impl HaCK for TestPlugin {
    fn name(&self) -> &str {
        "Test Plugin"
    }

    fn render_menu(&mut self, ui: &dyn UiBackend) {
        if ui.button("Toggle Window") {
            self.show_window = !self.show_window;
        }
    }
    fn render_window(&mut self, ui: &dyn UiBackend) {
        ui.text("Hello from Test Plugin!");
        if ui.button("Click Me!") {
            println!("Plugin button clicked!");
        }
    }

    impl_hac_boilerplate!(TestPlugin, hac_data);
}

struct FixedApp {
    hacks: Rc<RefCell<HaCKS>>,
}

impl eframe::App for FixedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hackers Runtime on Egui");

            // Create backend wrapper
            let backend = EguiBackend::new(ctx, ui);

            backend.separator();
            backend.text("Rendering Plugin Menu:");
            self.hacks.borrow_mut().render_menu(&backend);

            backend.separator();
            backend.text("Rendering Plugin Window:");
            if backend.collapsing_header("Test Plugin Window", TreeNodeFlags::default()) {
                // We need a backend for this inner UI!
                // But backend already wraps 'ui'. recursive?
                // backend.text works on the same ui.
                backend.text("Inner content placeholder for now");
            }

            backend.text("Note: Window rendering is stubbed in backend currently.");
        });

        // Sub-window 1
        egui::Window::new("Settings").show(ctx, |ui| {
            ui.label("Settings content here");
        });

        // Sub-window 2
        egui::Window::new("Tools")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.label("Tools content");
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    // Initialize HaCKS
    let hacks = Rc::new(RefCell::new(HaCKS::new()));

    // Register plugin (explicit cast/coercion to dyn HaCK)
    let plugin: Rc<RefCell<dyn HaCK>> = Rc::new(RefCell::new(TestPlugin::new()));
    hacks.borrow_mut().register_boxed(plugin);

    eframe::run_native(
        "Hackers Egui Standalone Example",
        options,
        Box::new(|_cc| Ok(Box::new(FixedApp { hacks }))),
    )
}
