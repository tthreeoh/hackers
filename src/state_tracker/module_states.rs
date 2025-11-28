// Replace src/state_tracker/module_states.rs
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

/// Lifecycle states that every module goes through
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HaCKLifecycleState {
    Uninitialized,
    Initializing,
    Ready,
    Updating,
    PostUpdate,
    RenderingMenu,
    PostRenderMenu,
    RenderingWindow,
    PostRenderWindow,
    RenderingDraw,
    PostRenderDraw,
    Unloading,
    Error,
    Qued,
    Stasis,
}

impl std::fmt::Display for HaCKLifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "Uninitialized"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Ready => write!(f, "Ready"),
            Self::Updating => write!(f, "Updating"),
            Self::PostUpdate => write!(f, "Post Update"),
            Self::RenderingMenu => write!(f, "Rendering Menu"),
            Self::PostRenderMenu => write!(f, "Post Rendering Menu"),
            Self::RenderingWindow => write!(f, "Rendering Window"),
            Self::PostRenderWindow => write!(f, "Post Rendering Window"),
            Self::RenderingDraw => write!(f, "Rendering Draw"),
            Self::PostRenderDraw => write!(f, "Post Rendering Draw"),
            Self::Unloading => write!(f, "Unloading"),
            Self::Error => write!(f, "Error"),
            Self::Qued => write!(f, "Qued"),
            Self::Stasis => write!(f, "Stasis"),
        }
    }
}

/// Custom state formatter for lifecycle states
#[derive(Default)]
pub struct LifecycleStateFormatter;

impl crate::state_tracker::statetracker::StateFormatter<HaCKLifecycleState> for LifecycleStateFormatter {
    fn format_state(&self, state: &HaCKLifecycleState) -> String {
        state.to_string()
    }
}