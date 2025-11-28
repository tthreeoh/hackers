
use std::any::TypeId;
use std::time::Instant;
use crate::{LifecycleStateFormatter, HaCKLifecycleState};
use crate::state_tracker::statetracker::StateTracker;

/// Wrapper that tracks state transitions for a module
pub struct TrackedModule {
    pub type_id: TypeId,
    pub name: String,
    pub lifecycle_tracker: StateTracker<HaCKLifecycleState, LifecycleStateFormatter>,
    pub last_update: Option<Instant>,
    pub update_count: u64,
    pub error_count: u64,
}

impl TrackedModule {
    pub fn new(type_id: TypeId, name: String) -> Self {
        let mut tracker = StateTracker::default();
        tracker.enabled = true;
        tracker.state_change(HaCKLifecycleState::Uninitialized);
        
        Self {
            type_id,
            name,
            lifecycle_tracker: tracker,
            last_update: None,
            update_count: 0,
            error_count: 0,
        }
    }

    pub fn begin_init(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Initializing);
    }

    pub fn end_init(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Ready);
    }

    pub fn begin_update(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Updating);
        self.update_count += 1;
    }

    pub fn end_update(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostUpdate);
        self.last_update = Some(Instant::now());
    }

    pub fn begin_render_menu(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::RenderingMenu);
    }

    pub fn end_render_menu(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostRenderMenu);
    }

    pub fn begin_render_window(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::RenderingWindow);
    }

    pub fn end_render_window(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostRenderWindow);
    }

    pub fn begin_render_draw(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::RenderingDraw);
    }

    pub fn end_render_draw(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostRenderDraw);
    }

    pub fn begin_unload(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Unloading);
    }

    pub fn mark_error(&mut self) {
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Error);
        self.error_count += 1;
    }

    pub fn reset_stats(&mut self) {
        self.lifecycle_tracker.reset();
        self.update_count = 0;
        self.error_count = 0;
        self.last_update = None;
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Ready);
    }
}