use crate::gui::UiBackend;
use crate::hackrs::HaCKS::HaCKS;
use crate::impl_backends::imgui_backend::ImguiBackend;

pub fn render_menu_bar(ui: &imgui::Ui, hacks: &mut HaCKS) {
    if let Some(_menu_bar) = ui.begin_main_menu_bar() {
        let backend = ImguiBackend::new(ui);
        hacks.render_menu(&backend);
    }
}

pub fn render_host_window(ui: &imgui::Ui, hacks: &mut HaCKS) {
    //Main inner window
    ui.window("Hackers Host Window")
        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(|| {
            about(ui, hacks);
        });
}

pub fn about(ui: &imgui::Ui, hacks: &mut HaCKS) {
    ui.text("Running on WGPU (DX12/Vulkan)");
    ui.separator();
    let io = ui.io();
    ui.text(format!("FPS: {:.1}", io.framerate));
    ui.separator();
    ui.text(format!("Loaded Modules: {}", hacks.hacs.len()));
}
