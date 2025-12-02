use crate::gui::{Color, Key, StyleColor, UiBackend};
use crate::metadata::HotkeyBinding;
use std::any::TypeId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::{HaCKS, HaCMetadata};

impl HaCKS {
    /// Sync all module hotkeys to the manager (call on init/module load)
    pub fn sync_hotkeys(&mut self) {
        for module_rc in self.hacs.values() {
            let module = module_rc.borrow();
            let type_id = module.nac_type_id();
            for binding in module.hotkey_bindings() {
                if let Some(hk) = binding.to_hotkey() {
                    let full_id = format!("{:?}::{}", type_id, binding.id);
                    self.hotkey_manager
                        .borrow_mut()
                        .register(full_id, hk, binding.cooldown());
                }
            }
        }
    }

    /// Dispatch triggered hotkeys to modules (call in render_draw)
    pub fn dispatch_hotkeys(&mut self, ui: &dyn UiBackend) {
        let triggered = self.hotkey_manager.borrow_mut().poll_all(ui);

        for full_id in triggered {
            // Parse "TypeId(...)::hotkey_name"
            if let Some((type_hash, hotkey_id)) = full_id.split_once("::") {
                // Find module by matching type_id debug string
                for tid in self.hacs.keys() {
                    if format!("{:?}", tid) == type_hash {
                        if let Some(module_rc) = self.hacs.get(tid) {
                            module_rc.borrow_mut().on_hotkey(hotkey_id);
                        }
                        break;
                    }
                }
            }
        }
    }
}

/// Trait for types that can be used as hotkey identifiers
pub trait HotkeyId: ToString + Clone + std::hash::Hash + Eq {
    fn as_str(&self) -> String {
        self.to_string()
    }
}

// Implement for String (default)
impl HotkeyId for String {}
impl HotkeyId for &str {}

/// Represents a single hotkey binding with optional modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

// Custom serialization for Key
mod key_serde {
    use crate::gui::Key;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[allow(unused)]
    pub fn serialize<S>(key: &Key, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (*key as i32).serialize(serializer)
    }

    #[allow(unused)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = i32::deserialize(deserializer)?;

        if value >= 0 && value <= 299 {
            Ok(unsafe { std::mem::transmute(value as u8) })
        } else {
            Err(D::Error::custom(format!("Invalid key value: {}", value)))
        }
    }
}

impl Hotkey {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            shift: false,
            ctrl: false,
            alt: false,
        }
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Check if this hotkey is currently pressed
    pub fn is_pressed(&self, ui: &dyn UiBackend) -> bool {
        let io = ui.io();
        let key_down = ui.is_key_down(self.key);
        let shift_match = !self.shift || io.key_shift;
        let ctrl_match = !self.ctrl || io.key_ctrl;
        let alt_match = !self.alt || io.key_alt;

        key_down && shift_match && ctrl_match && alt_match
    }
}

/// State tracking for a registered hotkey
#[derive(Debug, Clone, PartialEq)]
struct HotkeyState {
    hotkey: Hotkey,
    cooldown: Duration,
    last_trigger: Option<Instant>,
    was_pressed: bool,
}

// Custom serialization for Duration (serde doesn't support it by default)
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    #[allow(unused)]
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    #[allow(unused)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl HotkeyState {
    fn new(hotkey: Hotkey, cooldown: Duration) -> Self {
        Self {
            hotkey,
            cooldown,
            last_trigger: None,
            was_pressed: false,
        }
    }

    /// Check if enough time has passed since last trigger
    fn can_trigger(&self) -> bool {
        match self.last_trigger {
            None => true,
            Some(last) => last.elapsed() >= self.cooldown,
        }
    }

    /// Mark this hotkey as triggered
    fn trigger(&mut self) {
        self.last_trigger = Some(Instant::now());
    }
}

/// Manages multiple hotkeys with individual cooldowns and state tracking
#[derive(Debug, Clone, PartialEq)]
pub struct HotkeyManager {
    hotkeys: HashMap<String, HotkeyState>,
    capture_state: Option<String>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            hotkeys: HashMap::new(),
            capture_state: None,
        }
    }

    pub fn with_loader<F>(loader: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        let mut manager = Self::new();
        loader(&mut manager);
        manager
    }

    pub fn create_binding(id: impl Into<String>) -> HotkeyBinding {
        HotkeyBinding::unbound(id)
    }

    pub fn create_binding_with_key(id: impl Into<String>, key: Key) -> HotkeyBinding {
        HotkeyBinding::new(id, key)
    }

    pub fn generate_hotkey_id(existing: &[HotkeyBinding], prefix: &str) -> String {
        let mut counter = 0;
        loop {
            let id = format!("{}_{}", prefix, counter);
            if !existing.iter().any(|b| b.id == id) {
                return id;
            }
            counter += 1;
        }
    }

    pub fn find_conflict(&self, binding: &HotkeyBinding) -> Option<&str> {
        if let Some(hk) = binding.to_hotkey() {
            for (id, state) in &self.hotkeys {
                if state.hotkey == hk {
                    return Some(id);
                }
            }
        }
        None
    }

    pub fn find_hotkey_conflict(&self, hotkey: &Hotkey) -> Option<&str> {
        for (id, state) in &self.hotkeys {
            if &state.hotkey == hotkey {
                return Some(id);
            }
        }
        None
    }

    pub fn list_all(&self) -> Vec<(&str, &Hotkey)> {
        self.hotkeys
            .iter()
            .map(|(id, state)| (id.as_str(), &state.hotkey))
            .collect()
    }

    pub fn register<I>(&mut self, id: I, hotkey: Hotkey, cooldown: Duration)
    where
        I: ToString,
    {
        self.hotkeys
            .insert(id.to_string(), HotkeyState::new(hotkey, cooldown));
    }

    pub fn unregister<I>(&mut self, id: I) -> bool
    where
        I: ToString,
    {
        self.hotkeys.remove(&id.to_string()).is_some()
    }

    pub fn is_triggered<I>(&mut self, id: I, ui: &dyn UiBackend) -> bool
    where
        I: ToString,
    {
        let id_str = id.to_string();
        if let Some(state) = self.hotkeys.get_mut(&id_str) {
            let is_pressed = state.hotkey.is_pressed(ui);

            if is_pressed && !state.was_pressed && state.can_trigger() {
                state.trigger();
                state.was_pressed = true;
                return true;
            }

            state.was_pressed = is_pressed;
        }
        false
    }

    pub fn is_held<I>(&self, id: I, ui: &dyn UiBackend) -> bool
    where
        I: ToString,
    {
        if let Some(state) = self.hotkeys.get(&id.to_string()) {
            state.hotkey.is_pressed(ui)
        } else {
            false
        }
    }

    pub fn reset_cooldown<I>(&mut self, id: I)
    where
        I: ToString,
    {
        if let Some(state) = self.hotkeys.get_mut(&id.to_string()) {
            state.last_trigger = None;
        }
    }

    pub fn set_cooldown<I>(&mut self, id: I, cooldown: Duration)
    where
        I: ToString,
    {
        if let Some(state) = self.hotkeys.get_mut(&id.to_string()) {
            state.cooldown = cooldown;
        }
    }

    pub fn cooldown_remaining<I>(&self, id: I) -> Option<Duration>
    where
        I: ToString,
    {
        self.hotkeys.get(&id.to_string()).and_then(|state| {
            state.last_trigger.map(|last| {
                let elapsed = last.elapsed();
                if elapsed < state.cooldown {
                    state.cooldown - elapsed
                } else {
                    Duration::ZERO
                }
            })
        })
    }

    pub fn poll_all(&mut self, ui: &dyn UiBackend) -> Vec<String> {
        let ids: Vec<String> = self.hotkeys.keys().cloned().collect();
        ids.into_iter()
            .filter(|id| self.is_triggered(id, ui))
            .collect()
    }

    pub fn sync_from_bindings(&mut self, module_id: TypeId, hotkeys: &[HotkeyBinding]) {
        let prefix = format!("{:?}", module_id);

        let to_remove: Vec<_> = self
            .hotkeys
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in to_remove {
            self.hotkeys.remove(&key);
        }

        for binding in hotkeys {
            if let Some(hk) = binding.to_hotkey() {
                let full_id = format!("{}::{}", prefix, binding.id);
                self.register(full_id, hk, binding.cooldown());
            }
        }
    }

    pub fn sync_all<'a>(&mut self, modules: impl Iterator<Item = (TypeId, &'a [HotkeyBinding])>) {
        for (type_id, bindings) in modules {
            self.sync_from_bindings(type_id, bindings);
        }
    }

    pub fn module_hotkey_triggered(
        &mut self,
        module_id: TypeId,
        hotkey_id: &str,
        ui: &dyn UiBackend,
    ) -> bool {
        let full_id = format!("{:?}::{}", module_id, hotkey_id);
        self.is_triggered(&full_id, ui)
    }

    pub fn render_config(
        &mut self,
        ui: &dyn UiBackend,
        hotkeys: &mut Vec<HotkeyBinding>,
        global_manager: Option<&HotkeyManager>,
    ) -> bool {
        let mut modified = false;
        let mut to_remove: Option<usize> = None;

        for (idx, binding) in hotkeys.iter_mut().enumerate() {
            let id = binding.id.clone();

            let conflict = global_manager.and_then(|gm| gm.find_conflict(binding));

            ui.group(&mut || {
                let is_capturing = self
                    .capture_state
                    .as_ref()
                    .map(|s| s == &id)
                    .unwrap_or(false);

                let btn_label = if is_capturing {
                    format!("[ Press key... ]##{}", id)
                } else {
                    format!("{}##{}", Self::format_binding(binding), id)
                };

                let _token = conflict.map(|_| {
                    ui.push_style_color(StyleColor::Button, Color::new(0.8, 0.2, 0.2, 1.0))
                });

                if ui.button(&btn_label) {
                    self.capture_state = if is_capturing { None } else { Some(id.clone()) };
                }

                drop(_token);

                if is_capturing {
                    if let Some(key) = Self::detect_key(ui) {
                        binding.key = key as i32;
                        binding.shift = ui.io().key_shift;
                        binding.ctrl = ui.io().key_ctrl;
                        binding.alt = ui.io().key_alt;
                        self.capture_state = None;
                        modified = true;
                    }
                    if ui.is_key_pressed(Key::Escape) {
                        self.capture_state = None;
                    }
                }

                ui.same_line();
                ui.set_next_item_width(100.0);
                if ui.input_text(&format!("##{}_id", id), &mut binding.id) {
                    modified = true;
                }

                ui.same_line();
                if ui.checkbox(&format!("C##{}", id), &mut binding.ctrl) {
                    modified = true;
                }
                ui.same_line();
                if ui.checkbox(&format!("S##{}", id), &mut binding.shift) {
                    modified = true;
                }
                ui.same_line();
                if ui.checkbox(&format!("A##{}", id), &mut binding.alt) {
                    modified = true;
                }

                ui.same_line();
                ui.set_next_item_width(80.0);
                let mut cd = binding.cooldown_ms as i32;
                if ui.input_int_with_step(&format!("ms##{}", id), &mut cd, 50, 50) {
                    binding.cooldown_ms = cd.max(0) as u64;
                    modified = true;
                }

                ui.same_line();
                if ui.small_button(&format!("X##{}", id)) {
                    to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = to_remove {
            hotkeys.remove(idx);
            modified = true;
        }

        if ui.button("+ Add Hotkey") {
            let new_id = HotkeyManager::generate_hotkey_id(hotkeys, "hotkey");
            hotkeys.push(HotkeyBinding::unbound(new_id));
            modified = true;
        }

        modified
    }

    fn format_binding(b: &HotkeyBinding) -> String {
        if !b.is_bound() {
            return "[Unbound]".to_string();
        }
        let mut s = String::new();
        if b.ctrl {
            s.push_str("Ctrl+");
        }
        if b.shift {
            s.push_str("Shift+");
        }
        if b.alt {
            s.push_str("Alt+");
        }
        let key: Key = unsafe { std::mem::transmute(b.key as u8) };
        s.push_str(&format!("{:?}", key));
        s
    }

    fn detect_key(ui: &dyn UiBackend) -> Option<Key> {
        Self::detect_key_static(ui)
    }

    pub fn detect_key_static(ui: &dyn UiBackend) -> Option<Key> {
        const KEYS: &[Key] = &[
            Key::A,
            Key::B,
            Key::C,
            Key::D,
            Key::E,
            Key::F,
            Key::G,
            Key::H,
            Key::I,
            Key::J,
            Key::K,
            Key::L,
            Key::M,
            Key::N,
            Key::O,
            Key::P,
            Key::Q,
            Key::R,
            Key::S,
            Key::T,
            Key::U,
            Key::V,
            Key::W,
            Key::X,
            Key::Y,
            Key::Z,
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
            Key::F9,
            Key::F10,
            Key::F11,
            Key::F12,
            Key::Keypad0,
            Key::Keypad1,
            Key::Keypad2,
            Key::Keypad3,
            Key::Keypad4,
            Key::Keypad5,
            Key::Keypad6,
            Key::Keypad7,
            Key::Keypad8,
            Key::Keypad9,
            Key::Space,
            Key::Tab,
            Key::Enter,
            Key::Backspace,
            Key::Delete,
            Key::Insert,
            Key::Home,
            Key::End,
            Key::PageUp,
            Key::PageDown,
            Key::LeftArrow,
            Key::RightArrow,
            Key::UpArrow,
            Key::DownArrow,
            Key::GraveAccent,
        ];
        KEYS.iter().find(|&&k| ui.is_key_down(k)).copied()
    }
}

impl HaCKS {
    /// Sync all module hotkeys to the manager
    pub fn sync_all_hotkeys(&mut self) {
        let bindings: Vec<_> = self
            .hacs
            .iter()
            .map(|(&tid, m_rc)| {
                let m = m_rc.borrow();
                (tid, m.metadata().hotkeys.clone())
            })
            .collect();

        for (tid, hks) in bindings {
            self.hotkey_manager
                .borrow_mut()
                .sync_from_bindings(tid, &hks);
        }
    }

    /// Sync a single module's hotkeys (call after config UI changes)
    pub fn sync_module_hotkeys<T: 'static>(&mut self) {
        let tid = TypeId::of::<T>();
        if let Some(module_rc) = self.hacs.get(&tid) {
            let module = module_rc.borrow();
            let bindings = module.metadata().hotkeys.clone();
            self.hotkey_manager
                .borrow_mut()
                .sync_from_bindings(tid, &bindings);
        }
    }

    /// Get triggered hotkeys for a specific module this frame
    pub fn get_module_triggers(&self, module_id: TypeId) -> Vec<String> {
        let prefix = format!("{:?}::", module_id);
        self.triggered_hotkeys
            .borrow_mut()
            .iter()
            .filter_map(|full_id| full_id.strip_prefix(&prefix).map(|s| s.to_string()))
            .collect()
    }
}

impl HaCMetadata {
    /// Render hotkey config UI
    /// - `manager`: The HotkeyManager that owns capture state for this UI
    /// - `global`: Optional reference to app-level hotkeys for conflict detection
    pub fn render_hotkey_config(
        &mut self,
        ui: &dyn UiBackend,
        manager: &mut HotkeyManager,
        global: Option<&HotkeyManager>,
    ) -> bool {
        manager.render_config(ui, &mut self.hotkeys, global)
    }

    pub fn render_hotkey_config_simple(&mut self, ui: &dyn UiBackend) -> bool {
        use std::cell::RefCell;
        thread_local! {
            static CAPTURE: RefCell<Option<String>> = RefCell::new(None);
        }

        CAPTURE.with(|capture| {
            let mut capture = capture.borrow_mut();
            Self::render_config_inner(ui, &mut self.hotkeys, &mut capture)
        })
    }

    fn render_config_inner(
        ui: &dyn UiBackend,
        hotkeys: &mut Vec<HotkeyBinding>,
        capture_state: &mut Option<String>,
    ) -> bool {
        use crate::gui::hotkey_manager::HotkeyManager;

        let mut modified = false;
        let mut to_remove: Option<usize> = None;

        for (idx, binding) in hotkeys.iter_mut().enumerate() {
            let id = binding.id.clone();

            ui.group(&mut || {
                let is_capturing = capture_state.as_ref().map(|s| s == &id).unwrap_or(false);

                let btn_label = if is_capturing {
                    format!("[ Press key... ]##{}", id)
                } else {
                    format!("{}##{}", Self::format_binding(binding), id)
                };

                if ui.button(&btn_label) {
                    *capture_state = if is_capturing { None } else { Some(id.clone()) };
                }

                if is_capturing {
                    if let Some(key) = HotkeyManager::detect_key_static(ui) {
                        binding.key = key as i32;
                        binding.shift = ui.io().key_shift;
                        binding.ctrl = ui.io().key_ctrl;
                        binding.alt = ui.io().key_alt;
                        *capture_state = None;
                        modified = true;
                    }
                    if ui.is_key_pressed(Key::Escape) {
                        *capture_state = None;
                    }
                }

                ui.same_line();
                // Make ID editable
                ui.set_next_item_width(100.0);
                if ui.input_text(&format!("##{}_id", id), &mut binding.id) {
                    modified = true;
                }

                ui.same_line();
                if ui.checkbox(&format!("C##{}", id), &mut binding.ctrl) {
                    modified = true;
                }
                ui.same_line();
                if ui.checkbox(&format!("S##{}", id), &mut binding.shift) {
                    modified = true;
                }
                ui.same_line();
                if ui.checkbox(&format!("A##{}", id), &mut binding.alt) {
                    modified = true;
                }

                ui.same_line();
                ui.set_next_item_width(80.0);
                let mut cd = binding.cooldown_ms as i32;
                if ui.input_int_with_step(&format!("ms##{}", id), &mut cd, 50, 50) {
                    binding.cooldown_ms = cd.max(0) as u64;
                    modified = true;
                }

                ui.same_line();
                if ui.small_button(&format!("X##{}", id)) {
                    to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = to_remove {
            hotkeys.remove(idx);
            modified = true;
        }

        // Add new hotkey button
        if ui.button("+ Add Hotkey") {
            let new_id = HotkeyManager::generate_hotkey_id(hotkeys, "hotkey");
            hotkeys.push(HotkeyBinding::unbound(new_id));
            modified = true;
        }

        modified
    }

    fn format_binding(b: &HotkeyBinding) -> String {
        if !b.is_bound() {
            return "[Unbound]".to_string();
        }
        let mut s = String::new();
        if b.ctrl {
            s.push_str("Ctrl+");
        }
        if b.shift {
            s.push_str("Shift+");
        }
        if b.alt {
            s.push_str("Alt+");
        }
        let key: Key = unsafe { std::mem::transmute(b.key as u8) };
        s.push_str(&format!("{:?}", key));
        s
    }
}
