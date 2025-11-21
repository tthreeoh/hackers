use std::collections::HashMap;
use std::time::{Duration, Instant};
use imgui::{Ui, Key};
use serde::{Deserialize, Serialize};
use crate::metadata::HotkeyBinding;
use std::any::TypeId;

use crate::{HaCK, HaCKS, HaCMetadata};

impl HaCKS {
    /// Sync all module hotkeys to the manager (call on init/module load)
    pub fn sync_hotkeys(&mut self) {
        for module in self.hacs.values() {
            let type_id = module.nac_type_id();
            for binding in module.hotkey_bindings() {
                if let Some(hk) = binding.to_hotkey() {
                    let full_id = format!("{:?}::{}", type_id, binding.id);
                    self.hotkey_manager.register(full_id, hk, binding.cooldown());
                }
            }
        }
    }
    
    
    /// Dispatch triggered hotkeys to modules (call in render_draw)
    pub fn dispatch_hotkeys(&mut self, ui: &imgui::Ui) {
        let triggered = self.hotkey_manager.poll_all(ui);
        
        for full_id in triggered {
            // Parse "TypeId(...)::hotkey_name"
            if let Some((type_hash, hotkey_id)) = full_id.split_once("::") {
                // Find module by matching type_id debug string
                let type_ids: Vec<_> = self.hacs.keys().copied().collect();
                for tid in type_ids {
                    if format!("{:?}", tid) == type_hash {
                        if let Some(module) = self.hacs.get_mut(&tid) {
                            module.on_hotkey(hotkey_id);
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

// Custom serialization for imgui::Key
mod key_serde {
    use imgui::Key;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &Key, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (*key as i32).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = i32::deserialize(deserializer)?;
        
        // Safe conversion - validate the range
        if value >= 0 && value < 652 { // ImGui has ~652 keys
            Ok(unsafe { std::mem::transmute(value) })
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
    pub fn is_pressed(&self, ui: &Ui) -> bool {
        let io = ui.io();
        let key_down = io.keys_down[self.key as usize];
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

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

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
    capture_state: Option<String>,  // Add this field
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
    
    /// Create a new HotkeyManager with a loader function
    /// 
    /// # Example
    /// ```
    /// let manager = HotkeyManager::with_loader(|mgr| {
    ///     mgr.register("save", Hotkey::new(Key::F5), Duration::from_millis(500));
    ///     mgr.register("load", Hotkey::new(Key::F9), Duration::from_millis(500));
    /// });
    /// ```
    pub fn with_loader<F>(loader: F) -> Self 
    where 
        F: FnOnce(&mut Self)
    {
        let mut manager = Self::new();
        loader(&mut manager);
        manager
    }
    
    /// Register a new hotkey with a unique identifier
    /// 
    /// # Arguments
    /// * `id` - Unique identifier for this hotkey (can be String or enum implementing Display)
    /// * `hotkey` - The key combination to bind
    /// * `cooldown` - Minimum time between triggers (use Duration::ZERO for no cooldown)
    pub fn register<I>(&mut self, id: I, hotkey: Hotkey, cooldown: Duration) 
    where 
        I: ToString
    {
        self.hotkeys.insert(id.to_string(), HotkeyState::new(hotkey, cooldown));
    }
    
    /// Unregister a hotkey by its ID
    pub fn unregister<I>(&mut self, id: I) -> bool 
    where
        I: ToString
    {
        self.hotkeys.remove(&id.to_string()).is_some()
    }
    
    /// Check if a hotkey is currently pressed AND can be triggered (respects cooldown)
    /// This is the main polling function - call during render_draw
    /// 
    /// Returns true on the first frame the key is pressed (if cooldown allows)
    pub fn is_triggered<I>(&mut self, id: I, ui: &Ui) -> bool 
    where
        I: ToString
    {
        let id_str = id.to_string();
        if let Some(state) = self.hotkeys.get_mut(&id_str) {
            let is_pressed = state.hotkey.is_pressed(ui);
            
            // Edge detection: only trigger on press edge (not held)
            if is_pressed && !state.was_pressed && state.can_trigger() {
                state.trigger();
                state.was_pressed = true;
                return true;
            }
            
            state.was_pressed = is_pressed;
        }
        false
    }
    
    /// Check if a hotkey is currently being held down (no cooldown check)
    pub fn is_held<I>(&self, id: I, ui: &Ui) -> bool 
    where
        I: ToString
    {
        if let Some(state) = self.hotkeys.get(&id.to_string()) {
            state.hotkey.is_pressed(ui)
        } else {
            false
        }
    }
    
    /// Force reset the cooldown for a hotkey (makes it immediately triggerable)
    pub fn reset_cooldown<I>(&mut self, id: I) 
    where
        I: ToString
    {
        if let Some(state) = self.hotkeys.get_mut(&id.to_string()) {
            state.last_trigger = None;
        }
    }
    
    /// Change the cooldown duration for a registered hotkey
    pub fn set_cooldown<I>(&mut self, id: I, cooldown: Duration) 
    where
        I: ToString
    {
        if let Some(state) = self.hotkeys.get_mut(&id.to_string()) {
            state.cooldown = cooldown;
        }
    }
    
    /// Get the time remaining until a hotkey can be triggered again
    pub fn cooldown_remaining<I>(&self, id: I) -> Option<Duration> 
    where
        I: ToString
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
    
    /// Poll all registered hotkeys and return a list of IDs that were triggered
    /// Useful for systems that want to handle multiple hotkeys at once
    pub fn poll_all(&mut self, ui: &Ui) -> Vec<String> {
        let ids: Vec<String> = self.hotkeys.keys().cloned().collect();
        ids.into_iter()
            .filter(|id| self.is_triggered(id, ui))
            .collect()
    }

    pub fn sync_from_bindings(&mut self, module_id: TypeId, hotkeys: &[HotkeyBinding]) {
        let prefix = format!("{:?}", module_id);
        
        // Remove old bindings for this module
        let to_remove: Vec<_> = self.hotkeys
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        
        for key in to_remove {
            self.hotkeys.remove(&key);
        }
        
        // Register new bindings (skip unbound)
        for binding in hotkeys {
            if let Some(hk) = binding.to_hotkey() {
                let full_id = format!("{}::{}", prefix, binding.id);
                self.register(full_id, hk, binding.cooldown());
            }
        }
    }
    
    /// Sync all modules' hotkeys (call on init)
    pub fn sync_all<'a>(&mut self, modules: impl Iterator<Item = (TypeId, &'a [HotkeyBinding])>) {
        for (type_id, bindings) in modules {
            self.sync_from_bindings(type_id, bindings);
        }
    }
    
    /// Check if a module's hotkey was triggered this frame
    /// Returns the hotkey id without the module prefix
    pub fn module_hotkey_triggered(&mut self, module_id: TypeId, hotkey_id: &str, ui: &Ui) -> bool {
        let full_id = format!("{:?}::{}", module_id, hotkey_id);
        self.is_triggered(&full_id, ui)
    }
    
    /// Render config UI for a set of hotkey bindings
    /// Returns true if any binding was modified
    pub fn render_config(&mut self, ui: &Ui, hotkeys: &mut Vec<HotkeyBinding>) -> bool {
        let mut modified = false;
        let mut to_remove: Option<usize> = None;
        
        for (idx, binding) in hotkeys.iter_mut().enumerate() {
            let id = binding.id.clone();
            ui.group(|| {
                let is_capturing = self.capture_state.as_ref().map(|s| s == &id).unwrap_or(false);
                
                let btn_label = if is_capturing {
                    format!("[ Press key... ]##{}", id)
                } else {
                    format!("{}##{}", Self::format_binding(binding), id)
                };
                
                if ui.button(&btn_label) {
                    self.capture_state = if is_capturing { None } else { Some(id.clone()) };
                }
                
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
                ui.text(&id);
                
                ui.same_line();
                if ui.checkbox(format!("C##{}", id), &mut binding.ctrl) { modified = true; }
                ui.same_line();
                if ui.checkbox(format!("S##{}", id), &mut binding.shift) { modified = true; }
                ui.same_line();
                if ui.checkbox(format!("A##{}", id), &mut binding.alt) { modified = true; }
                
                ui.same_line();
                ui.set_next_item_width(80.0);
                let mut cd = binding.cooldown_ms as i32;
                if ui.input_int(format!("ms##{}", id), &mut cd).step(50).build() {
                    binding.cooldown_ms = cd.max(0) as u64;
                    modified = true;
                }
                
                ui.same_line();
                if ui.small_button(format!("X##{}", id)) {
                    to_remove = Some(idx);
                }
            });
        }
        
        if let Some(idx) = to_remove {
            hotkeys.remove(idx);
            modified = true;
        }
        
        if ui.button("+ Add") {
            hotkeys.push(HotkeyBinding::unbound(format!("action_{}", hotkeys.len())));
            modified = true;
        }
        
        modified
    }

    fn format_binding(b: &HotkeyBinding) -> String {
        if !b.is_bound() {
            return "[Unbound]".to_string();
        }
        let mut s = String::new();
        if b.ctrl { s.push_str("Ctrl+"); }
        if b.shift { s.push_str("Shift+"); }
        if b.alt { s.push_str("Alt+"); }
        let key: Key = unsafe { std::mem::transmute(b.key) };
        s.push_str(&format!("{:?}", key));
        s
    }
    
    fn detect_key(ui: &Ui) -> Option<Key> {
        let io = ui.io();
        const KEYS: &[Key] = &[
            Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H,
            Key::I, Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P,
            Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X,
            Key::Y, Key::Z,
            Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
            Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12,
            Key::Keypad0, Key::Keypad1, Key::Keypad2, Key::Keypad3, Key::Keypad4,
            Key::Keypad5, Key::Keypad6, Key::Keypad7, Key::Keypad8, Key::Keypad9,
            Key::Space, Key::Tab, Key::Enter, Key::Backspace, Key::Delete,
            Key::Insert, Key::Home, Key::End, Key::PageUp, Key::PageDown,
            Key::LeftArrow, Key::RightArrow, Key::UpArrow, Key::DownArrow,
            Key::GraveAccent,
        ];
        KEYS.iter().find(|&&k| io.keys_down[k as usize]).copied()
    }
}

impl HaCKS {
    /// Sync all module hotkeys to the manager
    pub fn sync_all_hotkeys(&mut self) {
        let bindings: Vec<_> = self.hacs
            .iter()
            .map(|(&tid, m)| (tid, m.metadata().hotkeys.clone()))
            .collect();
        
        for (tid, hks) in bindings {
            self.hotkey_manager.sync_from_bindings(tid, &hks);
        }
    }
    
    /// Sync a single module's hotkeys (call after config UI changes)
    pub fn sync_module_hotkeys<T: 'static>(&mut self) {
        let tid = TypeId::of::<T>();
        if let Some(module) = self.hacs.get(&tid) {
            let bindings = module.metadata().hotkeys.clone();
            self.hotkey_manager.sync_from_bindings(tid, &bindings);
        }
    }
    
    /// Get triggered hotkeys for a specific module this frame
    pub fn get_module_triggers(&self, module_id: TypeId) -> Vec<String> {
        let prefix = format!("{:?}::", module_id);
        self.triggered_hotkeys
            .iter()
            .filter_map(|full_id| {
                full_id.strip_prefix(&prefix).map(|s| s.to_string())
            })
            .collect()
    }
    
    pub fn render_hotkey_config_for<T: HaCK + 'static>(&mut self, ui: &Ui) -> bool {
        let tid = std::any::TypeId::of::<T>();
        if let Some(module) = self.hacs.get_mut(&tid) {
            let hotkeys = &mut module.metadata_mut().hotkeys;
            self.hotkey_manager.render_config(ui, hotkeys)
        } else {
            false
        }
    }

}

// Convenience on HaCMetadata (add to metadata.rs)
impl HaCMetadata {
    pub fn render_hotkey_config(&mut self, ui: &imgui::Ui, manager: &mut HotkeyManager) -> bool {
        manager.render_config(ui, &mut self.hotkeys)
    }
}