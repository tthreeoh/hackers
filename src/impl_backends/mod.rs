#[cfg(feature = "ui-imgui")]
pub mod imgui_backend;

#[cfg(feature = "ui-egui")]
pub mod egui_backend;

// Re-export the active backend
#[cfg(feature = "ui-imgui")]
pub use imgui_backend::ImguiBackend as DefaultBackend;

#[cfg(all(feature = "ui-egui", not(feature = "ui-imgui")))]
pub use egui_backend::EguiBackend as DefaultBackend;
