
use std::any::TypeId;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::{LifecycleStateFormatter, HaCKLifecycleState};
use crate::state_tracker::statetracker::StateTracker;

#[derive(Debug, Clone)]
pub struct PhaseTimingEntry {
    pub from_phase: HaCKLifecycleState,
    pub to_phase: HaCKLifecycleState,
    pub timestamp: Instant,
    pub duration_since_last: Duration,
}

pub struct TrackedModule {
    pub type_id: TypeId,
    pub name: String,
    pub lifecycle_tracker: StateTracker<HaCKLifecycleState, LifecycleStateFormatter>,
    pub last_update: Option<Instant>,
    pub update_count: u64,
    pub error_count: u64,
    pub phase_timings: VecDeque<PhaseTimingEntry>,
    pub max_phase_history: usize,
    pub last_phase_timestamp: Option<(HaCKLifecycleState, Instant)>,
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
            phase_timings: VecDeque::with_capacity(1000),
            max_phase_history: 1000,
            last_phase_timestamp: None,
        }
    }

    fn track_phase_transition(&mut self, to_phase: HaCKLifecycleState) {
        if let Some((from_phase, last_time)) = self.last_phase_timestamp {
            let now = Instant::now();
            let duration = now.duration_since(last_time);
            
            let entry = PhaseTimingEntry {
                from_phase,
                to_phase,
                timestamp: now,
                duration_since_last: duration,
            };
            
            // Manage history size
            if self.phase_timings.len() >= self.max_phase_history {
                self.phase_timings.pop_front();
            }
            
            self.phase_timings.push_back(entry);
            self.last_phase_timestamp = Some((to_phase, now));
        } else {
            self.last_phase_timestamp = Some((to_phase, Instant::now()));
        }
    }

    pub fn qued(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Qued);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Qued);
    }

    pub fn stasis(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Stasis);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Stasis);
    }

    pub fn begin_init(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Initializing);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Initializing);
    }

    pub fn end_init(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Ready);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Ready);
    }

    pub fn begin_update(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Updating);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Updating);
        self.update_count += 1;
    }

    pub fn end_update(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::PostUpdate);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostUpdate);
        self.last_update = Some(Instant::now());
    }

    pub fn begin_render_menu(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::RenderingMenu);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::RenderingMenu);
    }

    pub fn end_render_menu(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::PostRenderMenu);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostRenderMenu);
    }

    pub fn begin_render_window(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::RenderingWindow);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::RenderingWindow);
    }

    pub fn end_render_window(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::PostRenderWindow);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostRenderWindow);
    }

    pub fn begin_render_draw(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::RenderingDraw);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::RenderingDraw);
    }

    pub fn end_render_draw(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::PostRenderDraw);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::PostRenderDraw);
    }

    pub fn begin_unload(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Unloading);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Unloading);
    }

    pub fn mark_error(&mut self) {
        self.track_phase_transition(HaCKLifecycleState::Error);
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Error);
        self.error_count += 1;
    }

    pub fn reset_stats(&mut self) {
        self.lifecycle_tracker.reset();
        self.update_count = 0;
        self.error_count = 0;
        self.last_update = None;
        self.phase_timings.clear();
        self.last_phase_timestamp = Some((HaCKLifecycleState::Ready, Instant::now()));
        self.lifecycle_tracker.state_change(HaCKLifecycleState::Ready);
    }
    
    pub fn get_average_transition_time(&self, from: HaCKLifecycleState, to: HaCKLifecycleState) -> Option<Duration> {
        let transitions: Vec<Duration> = self.phase_timings
            .iter()
            .filter(|entry| entry.from_phase == from && entry.to_phase == to)
            .map(|entry| entry.duration_since_last)
            .collect();
        
        if transitions.is_empty() {
            return None;
        }
        
        let total: Duration = transitions.iter().sum();
        Some(total / transitions.len() as u32)
    }

    pub fn get_recent_transitions(&self, count: usize) -> Vec<&PhaseTimingEntry> {
        self.phase_timings
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    pub fn time_since_last_phase(&self, phase: HaCKLifecycleState) -> Option<Duration> {
        if let Some((current_phase, last_time)) = self.last_phase_timestamp {
            if current_phase == phase {
                return Some(last_time.elapsed());
            }
        }
        
        self.phase_timings
            .iter()
            .rev()
            .find(|entry| entry.to_phase == phase)
            .map(|entry| entry.timestamp.elapsed())
    }

    pub fn get_transition_statistics(&self) -> Vec<(HaCKLifecycleState, HaCKLifecycleState, Duration, u64)> {
        use std::collections::HashMap;
        
        let mut stats: HashMap<(HaCKLifecycleState, HaCKLifecycleState), (Duration, u64)> = HashMap::new();
        
        for entry in &self.phase_timings {
            let key = (entry.from_phase, entry.to_phase);
            let stat = stats.entry(key).or_insert((Duration::ZERO, 0));
            stat.0 += entry.duration_since_last;
            stat.1 += 1;
        }
        
        stats.into_iter()
            .map(|((from, to), (total, count))| {
                let avg = total / count.max(1) as u32;
                (from, to, avg, count)
            })
            .collect()
    }

    pub fn clear_phase_timings(&mut self) {
        self.phase_timings.clear();
        if let Some((current_phase, _)) = self.last_phase_timestamp {
            self.last_phase_timestamp = Some((current_phase, Instant::now()));
        }
    }

}