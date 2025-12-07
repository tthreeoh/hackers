use hackers::gui::UiBackend;
use hackers::hackrs::HaCKS::HaCKS;
use hackers::impl_backends::imgui_backend::ImguiBackend;

pub fn render_menu_bar(ui: &imgui::Ui, hacks: &mut HaCKS) {
    if let Some(_menu_bar) = ui.begin_main_menu_bar() {
        let backend = ImguiBackend::new(ui);
        hacks.render_menu(&backend);
    }
}

pub fn render_host_window(ui: &imgui::Ui, hacks: &mut HaCKS) {
    ui.window("Hackers Host Window")
        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text("Running on WGPU (DX12/Vulkan)");
            ui.separator();
            let io = ui.io();
            ui.text(format!("FPS: {:.1}", io.framerate));
            ui.separator();
            ui.text(format!("Loaded Plugins: {}", hacks.dynamic_modules.len()));
        });

    let backend = ImguiBackend::new(ui);
    hacks.render_window(&backend);
}
