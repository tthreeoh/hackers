// ============================================================
// File: src/ui/context.rs
// ============================================================

use std::any::Any;

use crate::gui::{Color, Vec2};

/// UI context passed to modules during rendering
/// This is backend-agnostic but can hold backend-specific data
pub struct UiContext {
    /// Backend-specific context (downcasted as needed)
    backend: Box<dyn Any>,
    /// Current font size
    font_size: f32,
    /// Available content region
    content_region: Vec2,
}

impl UiContext {
    pub fn new<T: Any>(backend: T) -> Self {
        Self {
            backend: Box::new(backend),
            font_size: 14.0,
            content_region: Vec2::new(0.0, 0.0),
        }
    }

    /// Get backend-specific context
    pub fn backend<T: 'static>(&self) -> Option<&T> {
        self.backend.downcast_ref::<T>()
    }

    /// Get mutable backend-specific context
    pub fn backend_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.backend.downcast_mut::<T>()
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    pub fn content_region(&self) -> Vec2 {
        self.content_region
    }

    pub fn set_content_region(&mut self, region: Vec2) {
        self.content_region = region;
    }
}
