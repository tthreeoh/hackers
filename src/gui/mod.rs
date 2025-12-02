pub mod editor;
pub mod render;
pub mod color;
pub mod hotkey_manager;
pub mod widgets;

pub use widgets::*;

use std::any::Any;

pub mod types;
pub mod context;
pub mod traits;

#[cfg(feature = "ui-imgui")]
pub mod imgui_backend;

#[cfg(feature = "ui-egui")]
pub mod egui_backend;

pub use types::*;
pub use context::*;
pub use traits::*;

// Re-export the active backend
#[cfg(feature = "ui-imgui")]
pub use imgui_backend::ImguiBackend as DefaultBackend;

#[cfg(feature = "ui-egui")]
pub use egui_backend::EguiBackend as DefaultBackend;
