
use std::any::TypeId;
use std::collections::HashMap;

use crate::{HaCKLifecycleState, TrackedModule};

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
    PhaseTiming,
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


//state supression

impl GlobalStateTracker {
    /// Suppress a specific state across all modules
    pub fn suppress_state_globally(&mut self, state: HaCKLifecycleState) {
        for tracker in self.module_trackers.values_mut() {
            tracker.lifecycle_tracker.suppress_state(state);
        }
    }
    
    /// Unsuppress a specific state across all modules
    pub fn unsuppress_state_globally(&mut self, state: HaCKLifecycleState) {
        for tracker in self.module_trackers.values_mut() {
            tracker.lifecycle_tracker.unsuppress_state(&state);
        }
    }
    
    /// Toggle suppression of a state across all modules
    pub fn toggle_suppress_state_globally(&mut self, state: HaCKLifecycleState) {
        for tracker in self.module_trackers.values_mut() {
            tracker.lifecycle_tracker.toggle_suppress_state(state);
        }
    }
    
    /// Enable/disable suppression globally
    pub fn set_suppress_enabled_globally(&mut self, enabled: bool) {
        for tracker in self.module_trackers.values_mut() {
            tracker.lifecycle_tracker.suppress_enabled = enabled;
        }
    }
    
    /// Clear all suppressions across all modules
    pub fn clear_suppressions_globally(&mut self) {
        for tracker in self.module_trackers.values_mut() {
            tracker.lifecycle_tracker.clear_suppressions();
        }
    }
    
    /// Preset: Suppress idle states (Stasis, Qued)
    pub fn suppress_idle_states(&mut self) {
        self.suppress_state_globally(HaCKLifecycleState::Stasis);
        self.suppress_state_globally(HaCKLifecycleState::Qued);
        self.set_suppress_enabled_globally(true);
    }
    
    /// Preset: Suppress post-render states
    pub fn suppress_post_states(&mut self) {
        self.suppress_state_globally(HaCKLifecycleState::PostUpdate);
        self.suppress_state_globally(HaCKLifecycleState::PostRenderMenu);
        self.suppress_state_globally(HaCKLifecycleState::PostRenderWindow);
        self.suppress_state_globally(HaCKLifecycleState::PostRenderDraw);
        self.set_suppress_enabled_globally(true);
    }
    
    /// Preset: Show only active work states (Updating, Rendering*)
    pub fn suppress_all_except_active(&mut self) {
        use HaCKLifecycleState::*;
        let suppress_list = vec![
            Uninitialized, Initializing, Ready, PostUpdate,
            PostRenderMenu, PostRenderWindow, PostRenderDraw,
            Unloading, Error, Qued, Stasis
        ];
        
        for state in suppress_list {
            self.suppress_state_globally(state);
        }
        self.set_suppress_enabled_globally(true);
    }
}