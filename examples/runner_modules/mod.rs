pub mod event_handler;
pub mod host_ui;
pub mod imgui_context;
pub mod plugin_ui;
pub mod wgpu_setup;

pub use event_handler::{handle_event, AppState};
pub use host_ui::{render_host_window, render_menu_bar};
pub use imgui_context::ImguiContext;
pub use plugin_ui::render_plugin_manager;
pub use wgpu_setup::WgpuRenderer;
