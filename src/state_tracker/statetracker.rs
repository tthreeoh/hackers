// Complete replacement for src/state_tracker/statetracker.rs
// All trait bounds are now consistent: T: Clone + PartialEq + Eq + Hash

use std::fmt::Debug;
use std::time::{Duration, Instant};
use std::collections::{VecDeque, HashSet};

/// Trait for state formatting - implements string conversion for states
pub trait StateFormatter<T> {
    fn format_state(&self, state: &T) -> String;
}

/// Default formatter uses Debug trait
impl<T: Debug> StateFormatter<T> for () {
    fn format_state(&self, state: &T) -> String {
        format!("{:?}", state)
    }
}

/// Comprehensive statistical tracking for a state
#[derive(Debug, Clone)]
pub struct StateAggregate<T: Clone + PartialEq> {
    pub state: T,
    pub total_duration: Duration,
    pub total_occurrences: u64,
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub mean_duration: Duration,
    pub variance_accumulator: f64,
}

impl<T: Clone + PartialEq> StateAggregate<T> {
    fn new(state: T) -> Self {
        Self {
            state,
            total_duration: Duration::ZERO,
            total_occurrences: 0,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            mean_duration: Duration::ZERO,
            variance_accumulator: 0.0,
        }
    }

    fn update(&mut self, new_duration: Duration) {
        if self.total_occurrences == 0 {
            self.first_seen = Instant::now();
        }
        self.total_duration += new_duration;
        self.total_occurrences += 1;
        
        // Online variance calculation (Welford's algorithm)
        let new_mean = self.total_duration.as_secs_f64() / self.total_occurrences as f64;
        let delta = new_duration.as_secs_f64() - new_mean;
        self.variance_accumulator += delta * delta;
        self.mean_duration = Duration::from_secs_f64(new_mean);
        self.last_seen = Instant::now();
    }

    /// Calculate standard deviation
    fn standard_deviation(&self) -> Option<Duration> {
        if self.total_occurrences < 2 {
            return None;
        }
        let variance = self.variance_accumulator / (self.total_occurrences as f64 - 1.0);
        Some(Duration::from_secs_f64(variance.sqrt()))
    }

    /// Detailed state statistics
    fn stats(&self) -> StateStatistics {
        StateStatistics {
            total_duration: self.total_duration,
            occurrences: self.total_occurrences,
            mean_duration: self.mean_duration,
            standard_deviation: self.standard_deviation(),
            first_seen: self.first_seen,
            last_seen: self.last_seen,
        }
    }
}

/// Structured state statistics for reporting
#[derive(Debug, Clone)]
pub struct StateStatistics {
    pub total_duration: Duration,
    pub occurrences: u64,
    pub mean_duration: Duration,
    pub standard_deviation: Option<Duration>,
    pub first_seen: Instant,
    pub last_seen: Instant,
}

/// Generic state tracking system with customizable formatting and state suppression
#[derive(Clone, Debug)]
pub struct StateTracker<T: Clone + PartialEq + Eq + std::hash::Hash, F: StateFormatter<T> = ()> {
    pub enabled: bool,
    pub dock: bool,
    pub show: bool,
    pub current_state: Option<T>,
    pub current_state_start: Option<Instant>,
    pub state_aggregates: VecDeque<StateAggregate<T>>,
    pub session_start: Option<Instant>,
    pub formatter: F,
    pub max_history: usize,
    pub active_duration: Duration,
    pub active_start: Option<Instant>,
    // State filtering
    pub suppressed_states: HashSet<T>,
    pub suppress_enabled: bool,
}

impl<T: Clone + PartialEq + Eq + std::hash::Hash, F: StateFormatter<T>> Default for StateTracker<T, F> 
where F: Default 
{
    fn default() -> Self {
        Self {
            enabled: false,
            dock: false,
            show: false,
            current_state: None,
            current_state_start: None,
            state_aggregates: VecDeque::with_capacity(1000),
            session_start: None,
            formatter: F::default(),
            max_history: 1000,
            active_duration: Duration::ZERO,
            active_start: None,
            suppressed_states: HashSet::new(),
            suppress_enabled: false,
        }
    }
}

impl<T: Clone + PartialEq + Eq + std::hash::Hash, F: StateFormatter<T>> StateTracker<T, F> {
    // ============ Basic Controls ============
    pub fn show(&mut self) {
        self.show = true;
    }

    pub fn hide(&mut self) {
        self.show = false;
    }

    pub fn toggle_show(&mut self) {
        self.show = !self.show;
    }

    pub fn toggle_dock(&mut self) {
        self.dock = !self.dock;
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.active_duration = Duration::ZERO;
        self.active_start = None;
        self.session_start = None;
        self.current_state = None;
        self.current_state_start = None;
        self.state_aggregates.clear();
        self.state_aggregates.reserve(self.max_history);
    }

    // ============ State Suppression Methods ============
    pub fn suppress_state(&mut self, state: T) {
        self.suppressed_states.insert(state);
    }
    
    pub fn unsuppress_state(&mut self, state: &T) {
        self.suppressed_states.remove(state);
    }
    
    pub fn toggle_suppress_state(&mut self, state: T) {
        if self.suppressed_states.contains(&state) {
            self.suppressed_states.remove(&state);
        } else {
            self.suppressed_states.insert(state);
        }
    }
    
    pub fn clear_suppressions(&mut self) {
        self.suppressed_states.clear();
    }
    
    pub fn is_state_suppressed(&self, state: &T) -> bool {
        self.suppress_enabled && self.suppressed_states.contains(state)
    }
    
    pub fn toggle_suppress_enabled(&mut self) {
        self.suppress_enabled = !self.suppress_enabled;
    }

    // ============ State Tracking ============
    pub fn state_change(&mut self, new_state: T) {
        if let Some(start) = self.active_start.take() {
            self.active_duration += start.elapsed();
        }
        
        if !self.enabled {
            self.enabled = true;
        }
        
        self.active_start = Some(Instant::now());
        
        if let (Some(start), Some(state)) = (self.current_state_start, self.current_state.clone()) {
            let elapsed = start.elapsed();
            
            // Find or create aggregate for this state
            let aggregate = self.state_aggregates
                .iter_mut()
                .find(|agg| agg.state == state);
            
            match aggregate {
                Some(agg) => {
                    agg.update(elapsed);
                },
                None => {
                    // Manage history size
                    if self.state_aggregates.len() >= self.max_history {
                        self.state_aggregates.pop_front();
                    }
                    let mut new_agg = StateAggregate::new(state);
                    new_agg.update(elapsed);
                    self.state_aggregates.push_back(new_agg);
                }
            }
        }
        
        self.current_state_start = Some(Instant::now());
        self.current_state = Some(new_state);
    }

    // ============ Statistics (Filtered) ============
    pub fn get_stats(&self) -> Vec<(T, Duration, u32, Option<Duration>)> {
        self.state_aggregates
            .iter()
            .filter(|agg| !self.is_state_suppressed(&agg.state))
            .map(|agg| (
                agg.state.clone(), 
                agg.total_duration, 
                agg.total_occurrences as u32, 
                agg.standard_deviation()
            ))
            .collect()
    }

    pub fn get_state_statistics(&self) -> Vec<(T, StateStatistics)> {
        self.state_aggregates
            .iter()
            .filter(|agg| !self.is_state_suppressed(&agg.state))
            .map(|agg| (agg.state.clone(), agg.stats()))
            .collect()
    }

    pub fn get_active_time(&self) -> Duration {
        let filtered_duration: Duration = self.state_aggregates
            .iter()
            .filter(|agg| !self.is_state_suppressed(&agg.state))
            .map(|agg| agg.total_duration)
            .sum();
        
        // Add current active segment if state is not suppressed
        let current_addition = if let Some(ref state) = self.current_state {
            if !self.is_state_suppressed(state) {
                self.active_start
                    .map(|start| start.elapsed())
                    .unwrap_or(Duration::ZERO)
            } else {
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        };
        
        filtered_duration + current_addition
    }

    // ============ Statistics (Unfiltered) ============
    pub fn get_all_stats(&self) -> Vec<(T, Duration, u32, Option<Duration>, bool)> {
        self.state_aggregates
            .iter()
            .map(|agg| (
                agg.state.clone(), 
                agg.total_duration, 
                agg.total_occurrences as u32, 
                agg.standard_deviation(),
                self.is_state_suppressed(&agg.state)
            ))
            .collect()
    }

    pub fn get_all_state_statistics(&self) -> Vec<(T, StateStatistics, bool)> {
        self.state_aggregates
            .iter()
            .map(|agg| (
                agg.state.clone(), 
                agg.stats(),
                self.is_state_suppressed(&agg.state)
            ))
            .collect()
    }

    pub fn get_total_active_time(&self) -> Duration {
        self.active_duration + 
            self.active_start
                .map(|start| start.elapsed())
                .unwrap_or(Duration::ZERO)
    }

    pub fn get_active_session_duration(&self) -> Duration {
        self.get_total_active_time()
    }

    pub fn get_total_session_duration(&self) -> Duration {
        self.session_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    // ============ Logging ============
    pub fn log_stats<L>(&self, log_fn: L) 
    where L: Fn(&str) {
        log_fn("State Duration Statistics:");
        log_fn(&format!("Total session time: {:?}", self.get_total_session_duration()));
        
        if self.suppress_enabled && !self.suppressed_states.is_empty() {
            log_fn(&format!("Filtered time: {:?}", self.get_active_time()));
            log_fn(&format!("Total time: {:?}", self.get_total_active_time()));
            log_fn(&format!("{} states suppressed", self.suppressed_states.len()));
        }
        
        for (state, stat) in self.get_state_statistics() {
            log_fn(&format!(
                "{:<15}: count={:<4} total={:?}, avg={:?}, std_dev={:?}",
                self.formatter.format_state(&state),
                stat.occurrences,
                stat.total_duration,
                stat.mean_duration,
                stat.standard_deviation.unwrap_or(Duration::ZERO)
            ));
        }
    }

    pub fn print_stats<L>(&self, log_fn: L)
    where L: Fn(&str) {
        self.log_stats(log_fn);
    }
}