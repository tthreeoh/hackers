use std::borrow::Cow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HaCMetadata {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub category: Cow<'static, str>,
    pub hotkey: Option<String>,
    pub menu_weight: f32,
    pub window_weight: f32,
    pub draw_weight: f32,
    pub update_weight: f32,
    pub visible_in_gui: bool,
    pub is_menu_enabled: bool,
    pub is_window_enabled: bool,
    pub is_render_enabled: bool,
    pub is_update_enabled: bool,
    #[serde(default = "default_window_pos")]
    pub window_pos: [f32; 2],
    #[serde(default = "default_window_size")]
    pub window_size: [f32; 2],
    pub auto_resize_window: bool,
}

impl Default for HaCMetadata {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed("unknown"),
            description: Cow::Borrowed("unknown"),
            category: Cow::Borrowed("unknown"),
            hotkey: None,
            menu_weight: 1.0,
            window_weight: 1.0,
            draw_weight: 1.0,
            update_weight: 1.0,
            visible_in_gui: false,
            is_menu_enabled: true,
            is_window_enabled: false,
            is_render_enabled: false,
            is_update_enabled: false,
            auto_resize_window: true,
            window_pos: default_window_pos(),
            window_size: default_window_size()
        }
    }
}

pub const fn default_window_pos() -> [f32; 2] {
    [0.0, 0.0]
}

pub const fn default_window_size() -> [f32; 2] {
    [0.0, 0.0]
}

