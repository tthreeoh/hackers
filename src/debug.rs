// debug.rs - Reusable logging utility with tag-based filtering
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
#[cfg(feature = "gui")]
use imgui::Ui;

// Global state for all logging
static LOG_STATE: Lazy<Arc<Mutex<LogState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(LogState::default()))
});

#[derive(Debug, Clone)]
struct LogState {
    // Track last message per tag to avoid duplicates
    last_messages: HashMap<String, (String, std::time::Instant)>,
    // Store all log entries
    entries: Vec<LogEntry>,
    // Configuration
    config: LogConfig,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            last_messages: HashMap::new(),
            entries: Vec::new(),
            config: LogConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub once_timeout_secs: u64,
    pub enabled_tags: HashMap<String, bool>,
    #[cfg(feature = "gui")]
    pub show_in_ui: bool,
    pub console_output: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            once_timeout_secs: 1,
            enabled_tags: HashMap::new(),
            #[cfg(feature = "gui")]
            show_in_ui: true,
            console_output: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: std::time::Instant,
    pub tag: String,
    pub level: LogLevel,
    pub message: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn color(&self) -> [f32; 4] {
        match self {
            Self::Debug => [0.7, 0.7, 0.7, 1.0],
            Self::Info => [0.5, 0.8, 1.0, 1.0],
            Self::Warning => [1.0, 0.7, 0.3, 1.0],
            Self::Error => [1.0, 0.3, 0.3, 1.0],
        }
    }
}

/// Main logging utility - can be used anywhere in your codebase
pub struct LogOnce;

impl LogOnce {
    /// Log a message once per tag (won't repeat until timeout)
    /// 
    /// # Examples
    /// ```
    /// LogOnce::log("autocast.rules", "Evaluating rule: Fire Wall");
    /// LogOnce::log("autocast.cooldown", "Skill on cooldown: 500ms remaining");
    /// ```
    pub fn log(tag: impl Into<String>, message: impl Into<String>) {
        Self::log_with_level(tag, LogLevel::Info, message);
    }
    
    /// Log with a specific level
    pub fn log_with_level(tag: impl Into<String>, level: LogLevel, message: impl Into<String>) {
        Self::log_internal(tag.into(), level, message.into(), HashMap::new(), true);
    }
    
    /// Log with additional context data
    pub fn log_with_context(
        tag: impl Into<String>,
        message: impl Into<String>,
        context: HashMap<String, String>
    ) {
        Self::log_internal(tag.into(), LogLevel::Info, message.into(), context, true);
    }
    
    /// Log every time (no deduplication)
    pub fn log_always(tag: impl Into<String>, message: impl Into<String>) {
        Self::log_internal(tag.into(), LogLevel::Info, message.into(), HashMap::new(), false);
    }
    
    /// Debug level logging
    pub fn debug(tag: impl Into<String>, message: impl Into<String>) {
        Self::log_internal(tag.into(), LogLevel::Debug, message.into(), HashMap::new(), true);
    }
    
    /// Warning level logging
    pub fn warn(tag: impl Into<String>, message: impl Into<String>) {
        Self::log_internal(tag.into(), LogLevel::Warning, message.into(), HashMap::new(), true);
    }
    
    /// Error level logging
    pub fn error(tag: impl Into<String>, message: impl Into<String>) {
        Self::log_internal(tag.into(), LogLevel::Error, message.into(), HashMap::new(), true);
    }
    
    // Internal implementation
    fn log_internal(
        tag: String,
        level: LogLevel,
        message: String,
        context: HashMap<String, String>,
        check_once: bool
    ) {
        if let Ok(mut state) = LOG_STATE.lock() {
            if !state.config.enabled {
                return;
            }
            
            // Check if tag is enabled
            if let Some(&enabled) = state.config.enabled_tags.get(&tag) {
                if !enabled {
                    return;
                }
            }
            
            // Check for duplicates if using "once" logic
            if check_once {
                let now = std::time::Instant::now();
                let key = format!("{}:{}", tag, message);
                
                if let Some((_, last_time)) = state.last_messages.get(&key) {
                    let last_time_copy = *last_time;
                    if now.duration_since(last_time_copy).as_secs() < state.config.once_timeout_secs {
                        return; // Skip duplicate
                    }
                }
                
                state.last_messages.insert(key, (message.clone(), now));
                
                // Clean up old entries
                let timeout_threshold = state.config.once_timeout_secs * 5;
                state.last_messages.retain(|_, (_, time)| {
                    now.duration_since(*time).as_secs() < timeout_threshold
                });
            }
            
            // Create entry
            let entry = LogEntry {
                timestamp: std::time::Instant::now(),
                tag: tag.clone(),
                level,
                message: message.clone(),
                context,
            };
            
            // Add to entries
            state.entries.push(entry);
            
            // Trim if needed
            if state.entries.len() > state.config.max_entries {
                let remove_count = state.entries.len() - state.config.max_entries;
                state.entries.drain(0..remove_count);
            }
            
            // Console output
            if state.config.console_output {
                let level_str = match level {
                    LogLevel::Debug => "DEBUG",
                    LogLevel::Info => "INFO",
                    LogLevel::Warning => "WARN",
                    LogLevel::Error => "ERROR",
                };
                log::info!("[{}][{}] {}", level_str, tag, message);
            }
        }
    }
    
    /// Enable/disable logging globally
    pub fn set_enabled(enabled: bool) {
        if let Ok(mut state) = LOG_STATE.lock() {
            state.config.enabled = enabled;
        }
    }
    
    /// Enable/disable specific tag
    pub fn set_tag_enabled(tag: impl Into<String>, enabled: bool) {
        if let Ok(mut state) = LOG_STATE.lock() {
            state.config.enabled_tags.insert(tag.into(), enabled);
        }
    }
    
    /// Check if tag is enabled
    pub fn is_tag_enabled(tag: &str) -> bool {
        LOG_STATE.lock()
            .ok()
            .and_then(|s| s.config.enabled_tags.get(tag).copied())
            .unwrap_or(true) // Default to enabled if not specified
    }
    
    /// Get all unique tags that have been used
    pub fn get_all_tags() -> Vec<String> {
        if let Ok(state) = LOG_STATE.lock() {
            let mut tags: Vec<String> = state.entries.iter()
                .map(|e| e.tag.clone())
                .collect();
            tags.sort();
            tags.dedup();
            tags
        } else {
            Vec::new()
        }
    }
    
    /// Clear all logs
    pub fn clear() {
        if let Ok(mut state) = LOG_STATE.lock() {
            state.entries.clear();
            state.last_messages.clear();
        }
    }
    
    /// Get configuration
    pub fn get_config() -> LogConfig {
        LOG_STATE.lock()
            .map(|s| s.config.clone())
            .unwrap_or_default()
    }
    
    /// Set configuration
    pub fn set_config(config: LogConfig) {
        if let Ok(mut state) = LOG_STATE.lock() {
            state.config = config;
        }
    }
    
    /// Get all log entries
    pub fn get_entries() -> Vec<LogEntry> {
        LOG_STATE.lock()
            .map(|s| s.entries.clone())
            .unwrap_or_default()
    }
    
    /// Get entries filtered by tag
    pub fn get_entries_by_tag(tag: &str) -> Vec<LogEntry> {
        LOG_STATE.lock()
            .map(|s| {
                s.entries.iter()
                    .filter(|e| e.tag == tag)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Render debug UI window
    #[cfg(feature = "gui")]
    pub fn render_window(ui: &Ui, show: &mut bool) {
        if !*show {
            return;
        }
        
        ui.window("Debug Logger")
            .size([900.0, 600.0], imgui::Condition::FirstUseEver)
            .opened(show)
            .build(|| {
                Self::render_ui(ui);
            });
    }
    
    /// Render debug UI (can be embedded in other windows)
    #[cfg(feature = "gui")]
    pub fn render_ui(ui: &Ui) {
        if let Ok(mut state) = LOG_STATE.lock() {
            // Configuration section
            if ui.collapsing_header("Settings", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                ui.checkbox("Enabled", &mut state.config.enabled);
                ui.checkbox("Console Output", &mut state.config.console_output);
                
                let mut max_entries = state.config.max_entries as i32;
                if ui.input_int("Max Entries", &mut max_entries).build() {
                    state.config.max_entries = max_entries.max(100) as usize;
                }
                
                let mut timeout = state.config.once_timeout_secs as i32;
                if ui.input_int("Once Timeout (sec)", &mut timeout).build() {
                    state.config.once_timeout_secs = timeout.max(1) as u64;
                }
            }
            
            ui.separator();
            
            // Tag management
            if ui.collapsing_header("Tag Filters", imgui::TreeNodeFlags::empty()) {
                let all_tags = Self::get_all_tags();
                
                if ui.button("Enable All Tags") {
                    for tag in &all_tags {
                        state.config.enabled_tags.insert(tag.clone(), true);
                    }
                }
                
                ui.same_line();
                
                if ui.button("Disable All Tags") {
                    for tag in &all_tags {
                        state.config.enabled_tags.insert(tag.clone(), false);
                    }
                }
                
                ui.separator();
                
                for tag in all_tags {
                    let mut enabled = state.config.enabled_tags
                        .get(&tag)
                        .copied()
                        .unwrap_or(true);
                    
                    if ui.checkbox(&format!("{}##tag_{}", tag, tag), &mut enabled) {
                        state.config.enabled_tags.insert(tag.clone(), enabled);
                    }
                    
                    // Show count
                    let count = state.entries.iter()
                        .filter(|e| e.tag == tag)
                        .count();
                    ui.same_line();
                    ui.text_colored([0.6, 0.6, 0.6, 1.0], format!("({})", count));
                }
            }
            
            ui.separator();
            
            // Stats and controls
            ui.text(format!("Total Entries: {} / {}", 
                state.entries.len(), 
                state.config.max_entries
            ));
            ui.text(format!("Unique Tags: {}", 
                state.entries.iter()
                    .map(|e| &e.tag)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            ));
            
            if ui.button("Clear All") {
                state.entries.clear();
                state.last_messages.clear();
            }
            
            ui.same_line();
            
            if ui.button("Copy to Clipboard") {
                let text = state.entries.iter()
                    .map(|e| format!("[{}][{:?}] {}", e.tag, e.level, e.message))
                    .collect::<Vec<_>>()
                    .join("\n");
                ui.set_clipboard_text(&text);
            }
            
            ui.same_line();
            
            if ui.button("Export to File") {
                // TODO: Implement file export
            }
            
            ui.separator();
            
            // Log entries display
            ui.child_window("log_entries")
                .size([0.0, 0.0])
                .build(|| {
                    // Display in reverse order (newest first)
                    for entry in state.entries.iter().rev() {
                        // Check if tag is filtered
                        if let Some(&enabled) = state.config.enabled_tags.get(&entry.tag) {
                            if !enabled {
                                continue;
                            }
                        }
                        
                        let color = entry.level.color();
                        
                        ui.text_colored(
                            [0.5, 0.5, 0.5, 1.0],
                            format!("[{:.2}s]", entry.timestamp.elapsed().as_secs_f32())
                        );
                        
                        ui.same_line();
                        ui.text_colored(
                            [0.7, 0.9, 1.0, 1.0],
                            format!("[{}]", entry.tag)
                        );
                        
                        ui.same_line();
                        ui.text_colored(color, &entry.message);
                        
                        // Show context on hover
                        if !entry.context.is_empty() && ui.is_item_hovered() {
                            ui.tooltip(|| {
                                for (key, value) in &entry.context {
                                    ui.text(format!("{}: {}", key, value));
                                }
                            });
                        }
                    }
                    
                    if state.entries.is_empty() {
                        ui.text_colored([0.5, 0.5, 0.5, 1.0], "No log entries");
                    }
                });
        }
    }
}

// Convenience macros for common logging patterns
#[macro_export]
macro_rules! log_once {
    ($tag:expr, $($arg:tt)*) => {
        $crate::debug::LogOnce::log($tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($tag:expr, $($arg:tt)*) => {
        $crate::debug::LogOnce::debug($tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($tag:expr, $($arg:tt)*) => {
        $crate::debug::LogOnce::warn($tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($tag:expr, $($arg:tt)*) => {
        $crate::debug::LogOnce::error($tag, format!($($arg)*))
    };
}

// Helper for creating context maps
#[macro_export]
macro_rules! log_context {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}