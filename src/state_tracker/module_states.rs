use std::fmt::Debug;
use serde::{Deserialize, Serialize};

/// Lifecycle states that every module goes through
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleLifecycleState {
    Uninitialized,
    Initializing,
    Ready,
    Updating,
    RenderingMenu,
    RenderingWindow,
    RenderingDraw,
    Idle,
    Unloading,
    Error,
}

impl std::fmt::Display for ModuleLifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "Uninitialized"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Ready => write!(f, "Ready"),
            Self::Updating => write!(f, "Updating"),
            Self::RenderingMenu => write!(f, "Rendering Menu"),
            Self::RenderingWindow => write!(f, "Rendering Window"),
            Self::RenderingDraw => write!(f, "Rendering Draw"),
            Self::Idle => write!(f, "Idle"),
            Self::Unloading => write!(f, "Unloading"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Custom state formatter for lifecycle states
#[derive(Default)]
pub struct LifecycleStateFormatter;

impl crate::state_tracker::statetracker::StateFormatter<ModuleLifecycleState> for LifecycleStateFormatter {
    fn format_state(&self, state: &ModuleLifecycleState) -> String {
        state.to_string()
    }
}