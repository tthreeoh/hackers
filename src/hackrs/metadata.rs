use std::borrow::Cow;
use std::time::Duration;
use imgui::Key;
use serde::{Deserialize, Serialize};
use crate::access::{AccessControl, AccessLevel};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HotkeyBinding {
    pub id: String,
    pub key: i32,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub cooldown_ms: u64,
}

impl HotkeyBinding {
    pub fn new(id: impl Into<String>, key: Key) -> Self {
        Self {
            id: id.into(),
            key: key as i32,
            shift: false,
            ctrl: false,
            alt: false,
            cooldown_ms: 200,
        }
    }
    
    pub fn unbound(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key: -1,
            shift: false,
            ctrl: false,
            alt: false,
            cooldown_ms: 200,
        }
    }
    
    pub fn is_bound(&self) -> bool {
        self.key >= 0
    }
    
    pub fn with_shift(mut self) -> Self { self.shift = true; self }
    pub fn with_ctrl(mut self) -> Self { self.ctrl = true; self }
    pub fn with_alt(mut self) -> Self { self.alt = true; self }
    pub fn with_cooldown(mut self, ms: u64) -> Self { self.cooldown_ms = ms; self }
    
    pub fn to_hotkey(&self) -> Option<crate::gui::hotkey_manager::Hotkey> {
        if !self.is_bound() {
            return None;
        }
        let key: Key = unsafe { std::mem::transmute(self.key) };
        let mut hk = crate::gui::hotkey_manager::Hotkey::new(key);
        if self.shift { hk = hk.with_shift(); }
        if self.ctrl { hk = hk.with_ctrl(); }
        if self.alt { hk = hk.with_alt(); }
        Some(hk)
    }
    
    pub fn cooldown(&self) -> Duration {
        Duration::from_millis(self.cooldown_ms)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HaCMetadata {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub category: Cow<'static, str>,
    #[serde(default)]
    pub hotkeys: Vec<HotkeyBinding>,
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
    #[serde(default)]
    pub access_control: AccessControl,
}

impl Default for HaCMetadata {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed("unknown"),
            description: Cow::Borrowed("unknown"),
            category: Cow::Borrowed("unknown"),
            hotkeys: Vec::new(),
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
            window_size: default_window_size(),
            access_control: AccessControl::new(AccessLevel::ReadWrite)
        }
    }
}

pub const fn default_window_pos() -> [f32; 2] { [0.0, 0.0] }
pub const fn default_window_size() -> [f32; 2] { [0.0, 0.0] }