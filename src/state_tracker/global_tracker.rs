
use std::any::TypeId;
use std::collections::HashMap;

use crate::TrackedModule;

/// Global state tracking configuration
pub struct GlobalStateTracker {
    pub enabled: bool,
    pub show_window: bool,
    pub module_trackers: HashMap<TypeId, TrackedModule>,
    pub selected_module: Option<TypeId>,
    pub view_mode: StateViewMode,
    pub flatten_threshold: usize, // Flatten stats after N state changes
    pub auto_flatten: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateViewMode {
    AllModules,
    SelectedModule,
    Lifecycle,
    Performance,
}

impl Default for GlobalStateTracker {
    fn default() -> Self {
        Self {
            enabled: false,
            show_window: false,
            module_trackers: HashMap::new(),
            selected_module: None,
            view_mode: StateViewMode::AllModules,
            flatten_threshold: 10000,
            auto_flatten: true,
        }
    }
}

impl GlobalStateTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_module(&mut self, type_id: TypeId, name: String) {
        self.module_trackers.insert(
            type_id,
            TrackedModule::new(type_id, name),
        );
    }

    pub fn unregister_module(&mut self, type_id: &TypeId) {
        self.module_trackers.remove(type_id);
        if self.selected_module == Some(*type_id) {
            self.selected_module = None;
        }
    }

    pub fn get_tracker_mut(&mut self, type_id: &TypeId) -> Option<&mut TrackedModule> {
        self.module_trackers.get_mut(type_id)
    }

    pub fn get_tracker(&self, type_id: &TypeId) -> Option<&TrackedModule> {
        self.module_trackers.get(type_id)
    }

    pub fn reset_all(&mut self) {
        for tracker in self.module_trackers.values_mut() {
            tracker.reset_stats();
        }
    }

    pub fn reset_module(&mut self, type_id: &TypeId) {
        if let Some(tracker) = self.module_trackers.get_mut(type_id) {
            tracker.reset_stats();
        }
    }

    /// Flatten statistics for modules exceeding threshold
    pub fn flatten_if_needed(&mut self) {
        if !self.auto_flatten {
            return;
        }

        for tracker in self.module_trackers.values_mut() {
            let total_changes = tracker.lifecycle_tracker.state_aggregates.len();
            if total_changes > self.flatten_threshold {
                // Keep only the most recent N/2 entries
                let keep = self.flatten_threshold / 2;
                let remove = total_changes - keep;
                for _ in 0..remove {
                    tracker.lifecycle_tracker.state_aggregates.pop_front();
                }
            }
        }
    }
}