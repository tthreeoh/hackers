use imgui::Ui;

use crate::{HaCKLifecycleState, TrackedModule};

#[cfg(feature = "gui")]
pub mod gui_integration {
    use imgui::Ui;
    use crate::{GlobalStateTracker, HaCKLifecycleState, StateViewMode, state_tracker::ux_statetracker::{
        ProgressBarStateRenderer, RenderableStateTracker
    }};

    impl GlobalStateTracker {
        pub fn render_window(&mut self, ui: &Ui) {
            if !self.show_window {
                return;
            }

            let mut show = self.show_window;
            ui.window("State Tracker")
                .opened(&mut show)
                .size([800.0, 600.0], imgui::Condition::FirstUseEver)
                .menu_bar(true)
                .build(|| {
                    self.render_menu_bar(ui);
                    self.render_controls(ui);
                    ui.separator();
                    self.render_content(ui);
                });

            self.show_window = show;
        }

        fn render_menu_bar(&mut self, ui: &Ui) {
            if let Some(_menu_bar) = ui.begin_menu_bar() {
                ui.menu("View", || {
                    if ui.menu_item("All Modules") {
                        self.view_mode = StateViewMode::AllModules;
                    }
                    if ui.menu_item("Selected Module") {
                        self.view_mode = StateViewMode::SelectedModule;
                    }
                    if ui.menu_item("Lifecycle View") {
                        self.view_mode = StateViewMode::Lifecycle;
                    }
                    if ui.menu_item("Performance View") {
                        self.view_mode = StateViewMode::Performance;
                    }
                });
    
                ui.menu("Filter", || {
                    // Global suppression toggle
                    let mut any_suppressed = self.module_trackers.values()
                        .any(|t| t.lifecycle_tracker.suppress_enabled);
                    
                    if ui.checkbox("Enable Suppression", &mut any_suppressed) {
                        self.set_suppress_enabled_globally(any_suppressed);
                    }
                    
                    ui.separator();
                    
                    // Presets
                    if ui.menu_item("Suppress Idle States (Stasis, Qued)") {
                        self.suppress_idle_states();
                    }
                    
                    if ui.menu_item("Suppress Post-* States") {
                        self.suppress_post_states();
                    }
                    
                    if ui.menu_item("Show Only Active Work") {
                        self.suppress_all_except_active();
                    }
                    
                    ui.separator();
                    
                    // Individual state toggles
                    ui.text("Toggle Individual States:");
                    ui.separator();
                    
                    use HaCKLifecycleState::*;
                    let states = vec![
                        Uninitialized, Initializing, Ready, Updating, PostUpdate,
                        RenderingMenu, PostRenderMenu, RenderingWindow, PostRenderWindow,
                        RenderingDraw, PostRenderDraw, Unloading, Error, Qued, Stasis
                    ];
                    
                    for state in states {
                        // Check if any module has this state suppressed
                        let some_suppressed = self.module_trackers.values()
                            .any(|t| t.lifecycle_tracker.is_state_suppressed(&state));
                        
                        let label = format!("{}", state);
                        let mut is_suppressed = some_suppressed;
                        
                        if ui.checkbox(&label, &mut is_suppressed) {
                            if is_suppressed {
                                self.suppress_state_globally(state);
                            } else {
                                self.unsuppress_state_globally(state);
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    if ui.menu_item("Clear All Suppressions") {
                        self.clear_suppressions_globally();
                    }
                });
    
                ui.menu("Actions", || {
                    if ui.menu_item("Reset All Stats") {
                        self.reset_all();
                    }
                    if ui.menu_item("Flatten Statistics") {
                        self.flatten_if_needed();
                    }
                    if ui.menu_item("Close") {
                        self.show_window = false;
                    }
                });
            }
        }

        fn render_controls(&mut self, ui: &Ui) {
            ui.checkbox("Enable Tracking", &mut self.enabled);
            ui.same_line();
            ui.checkbox("Auto-flatten", &mut self.auto_flatten);

            if self.auto_flatten {
                ui.same_line();
                ui.set_next_item_width(100.0);
                let mut threshold = self.flatten_threshold as i32;
                if ui.input_int("Threshold", &mut threshold).build() {
                    self.flatten_threshold = threshold.max(100) as usize;
                }
            }

            // Module selector
            if let Some(_combo) = ui.begin_combo("Module", self.get_selected_module_name()) {
                if ui.selectable("(All Modules)") {
                    self.selected_module = None;
                }

                let mut sorted_modules: Vec<_> = self.module_trackers.iter().collect();
                sorted_modules.sort_by_key(|(_, tracker)| &tracker.name);

                for (type_id, tracker) in sorted_modules {
                    let is_selected = self.selected_module == Some(*type_id);
                    if ui.selectable_config(&tracker.name)
                        .selected(is_selected)
                        .build()
                    {
                        self.selected_module = Some(*type_id);
                    }
                }
            }
        }

        fn get_selected_module_name(&self) -> &str {
            self.selected_module
                .and_then(|id| self.module_trackers.get(&id))
                .map(|t| t.name.as_str())
                .unwrap_or("(All Modules)")
        }

        fn render_content(&mut self, ui: &Ui) {
            match self.view_mode {
                StateViewMode::AllModules => self.render_all_modules(ui),
                StateViewMode::SelectedModule => self.render_selected_module(ui),
                StateViewMode::Lifecycle => self.render_lifecycle_view(ui),
                StateViewMode::Performance => self.render_performance_view(ui),
                StateViewMode::PhaseTiming => self.render_phase_timing_view(ui),
            }
        }

        fn render_all_modules(&mut self, ui: &Ui) {
            let renderer = ProgressBarStateRenderer;
            
            let mut sorted: Vec<_> = self.module_trackers.iter_mut().collect();
            sorted.sort_by_key(|(_, t)| t.name.clone());

            for (_type_id, tracker) in sorted {
                if ui.collapsing_header(&tracker.name, imgui::TreeNodeFlags::empty()) {
                    ui.indent();
                    
                    ui.text(format!("Updates: {}", tracker.update_count));
                    ui.same_line();
                    ui.text(format!("Errors: {}", tracker.error_count));
                    
                    if let Some(last) = tracker.last_update {
                        ui.text(format!("Last update: {:.2}s ago", last.elapsed().as_secs_f32()));
                    }
                    
                    ui.separator();
                    tracker.lifecycle_tracker.render_stats(ui, &renderer);
                    
                    ui.unindent();
                }
            }
        }

        fn render_selected_module(&mut self, ui: &Ui) {
            if let Some(type_id) = self.selected_module {
                if let Some(tracker) = self.module_trackers.get_mut(&type_id) {
                    let renderer = ProgressBarStateRenderer;
                    
                    ui.text(format!("Module: {}", tracker.name));
                    ui.text(format!("Update Count: {}", tracker.update_count));
                    ui.text(format!("Error Count: {}", tracker.error_count));
                    
                    if let Some(last) = tracker.last_update {
                        ui.text(format!("Last Update: {:.2}s ago", last.elapsed().as_secs_f32()));
                    }
                    
                    ui.separator();
                    
                    // Show both filtered and total time when suppression is enabled
                    if tracker.lifecycle_tracker.suppress_enabled {
                        let filtered = tracker.lifecycle_tracker.get_active_time();
                        let total = tracker.lifecycle_tracker.get_total_active_time();
                        
                        ui.text(format!("Active Time (Filtered): {:.2}s", filtered.as_secs_f32()));
                        ui.text(format!("Active Time (Total): {:.2}s", total.as_secs_f32()));
                        
                        let suppressed_count = tracker.lifecycle_tracker.suppressed_states.len();
                        if suppressed_count > 0 {
                            ui.text_colored(
                                [0.7, 0.7, 0.0, 1.0],
                                &format!("({} states suppressed)", suppressed_count)
                            );
                        }
                    } else {
                        ui.text(format!(
                            "Active Session: {:.2}s",
                            tracker.lifecycle_tracker.get_active_session_duration().as_secs_f32()
                        ));
                    }
                    
                    ui.separator();
                    tracker.lifecycle_tracker.render_stats(ui, &renderer);
                    
                    if ui.button("Reset Stats") {
                        tracker.reset_stats();
                    }
                } else {
                    ui.text("Module not found");
                }
            } else {
                ui.text("No module selected");
            }
        }

        fn render_lifecycle_view(&self, ui: &Ui) {
            use std::collections::HashMap;
            use std::time::Duration;

            // Aggregate lifecycle stats across all modules
            let mut lifecycle_stats: HashMap<HaCKLifecycleState, (Duration, u64)> = HashMap::new();

            for tracker in self.module_trackers.values() {
                for (state, stats) in tracker.lifecycle_tracker.get_state_statistics() {
                    let entry = lifecycle_stats.entry(state).or_insert((Duration::ZERO, 0));
                    entry.0 += stats.total_duration;
                    entry.1 += stats.occurrences;
                }
            }

            ui.text("Aggregated Lifecycle Statistics");
            ui.separator();

            if let Some(_table) = ui.begin_table_with_flags(
                "lifecycle_table",
                3,
                imgui::TableFlags::BORDERS | imgui::TableFlags::ROW_BG,
            ) {
                ui.table_setup_column("State");
                ui.table_setup_column("Total Time");
                ui.table_setup_column("Occurrences");
                ui.table_headers_row();

                for (state, (duration, count)) in lifecycle_stats.iter() {
                    ui.table_next_row();
                    ui.table_next_column();
                    ui.text(format!("{}", state));
                    ui.table_next_column();
                    ui.text(format!("{:.2}s", duration.as_secs_f32()));
                    ui.table_next_column();
                    ui.text(format!("{}", count));
                }
            }
        }

        fn render_performance_view(&self, ui: &Ui) {
            ui.text("Performance Metrics");
            ui.separator();

            let mut sorted: Vec<_> = self.module_trackers.iter().collect();
            sorted.sort_by(|a, b| {
                b.1.lifecycle_tracker.get_active_session_duration()
                    .cmp(&a.1.lifecycle_tracker.get_active_session_duration())
            });

            if let Some(_table) = ui.begin_table_with_flags(
                "perf_table",
                4,
                imgui::TableFlags::BORDERS | imgui::TableFlags::ROW_BG,
            ) {
                ui.table_setup_column("Module");
                ui.table_setup_column("Active Time");
                ui.table_setup_column("Updates");
                ui.table_setup_column("Errors");
                ui.table_headers_row();

                for (_type_id, tracker) in sorted.iter().take(20) {
                    ui.table_next_row();
                    ui.table_next_column();
                    ui.text(&tracker.name);
                    ui.table_next_column();
                    ui.text(format!(
                        "{:.2}s",
                        tracker.lifecycle_tracker.get_active_session_duration().as_secs_f32()
                    ));
                    ui.table_next_column();
                    ui.text(format!("{}", tracker.update_count));
                    ui.table_next_column();
                    ui.text(format!("{}", tracker.error_count));
                }
            }
        }
                
        fn render_phase_timing_view(&self, ui: &Ui) {
            ui.text("Phase Transition Timings");
            ui.separator();

            let mut sorted: Vec<_> = self.module_trackers.iter().collect();
            sorted.sort_by_key(|(_, t)| t.name.clone());

            for (_type_id, tracker) in sorted.iter() {
                if ui.collapsing_header(&tracker.name, imgui::TreeNodeFlags::empty()) {
                    ui.indent();
                    
                    // Show recent transitions
                    let recent = tracker.get_recent_transitions(5);
                    if !recent.is_empty() {
                        ui.text("Recent Transitions:");
                        for entry in recent {
                            ui.text(format!(
                                "  {} → {} ({:.2}ms ago, took {:.2}ms)",
                                entry.from_phase,
                                entry.to_phase,
                                entry.timestamp.elapsed().as_millis(),
                                entry.duration_since_last.as_micros() as f32 / 1000.0
                            ));
                        }
                        ui.separator();
                    }

                    // Show transition statistics
                    let transition_stats = tracker.get_transition_statistics();
                    if !transition_stats.is_empty() {
                        ui.text("Transition Averages:");
                        
                        if let Some(_table) = ui.begin_table_with_flags(
                            &format!("trans_table_{}", tracker.name),
                            4,
                            imgui::TableFlags::BORDERS | imgui::TableFlags::ROW_BG,
                        ) {
                            ui.table_setup_column("From");
                            ui.table_setup_column("To");
                            ui.table_setup_column("Avg Time");
                            ui.table_setup_column("Count");
                            ui.table_headers_row();

                            for (from, to, avg_duration, count) in transition_stats {
                                ui.table_next_row();
                                ui.table_next_column();
                                ui.text(format!("{}", from));
                                ui.table_next_column();
                                ui.text(format!("{}", to));
                                ui.table_next_column();
                                if avg_duration.as_millis() < 1 {
                                    ui.text(format!("{:.2}μs", avg_duration.as_micros()));
                                } else {
                                    ui.text(format!("{:.2}ms", avg_duration.as_millis()));
                                }
                                ui.table_next_column();
                                ui.text(format!("{}", count));
                            }
                        }
                    }

                    // Time since last update cycle
                    if let Some(duration) = tracker.time_since_last_phase(HaCKLifecycleState::Updating) {
                        ui.separator();
                        ui.text(format!(
                            "Time since last update: {:.2}s",
                            duration.as_secs_f32()
                        ));
                    }

                    ui.unindent();
                }
            }
        }

    }
    
}


pub struct PhaseTimingVisualizer;

impl PhaseTimingVisualizer {
    /// Render a timeline of recent phase transitions
    pub fn render_timeline(ui: &Ui, tracker: &TrackedModule, max_entries: usize) {
        let recent = tracker.get_recent_transitions(max_entries);
        
        if recent.is_empty() {
            ui.text("No phase transitions recorded yet.");
            return;
        }

        ui.text("Phase Transition Timeline:");
        ui.separator();

        // Calculate max duration for scaling
        let max_duration = recent.iter()
            .map(|e| e.duration_since_last.as_micros())
            .max()
            .unwrap_or(1) as f32;

        let available_width = ui.content_region_avail()[0] - 20.0;
        
        for (i, entry) in recent.iter().enumerate() {
            let duration_ms = entry.duration_since_last.as_micros() as f32 / 1000.0;
            let duration_us = entry.duration_since_last.as_micros();
            
            // Calculate bar width proportional to duration
            let bar_width = if max_duration > 0.0 {
                (duration_us as f32 / max_duration) * available_width
            } else {
                available_width
            };

            // Color based on phase type
            let color = Self::phase_color(&entry.to_phase);
            
            // Draw the bar
            let cursor_pos = ui.cursor_screen_pos();
            let draw_list = ui.get_window_draw_list();
            
            let bar_height = 20.0;
            let p1 = [cursor_pos[0], cursor_pos[1]];
            let p2 = [cursor_pos[0] + bar_width, cursor_pos[1] + bar_height];
            
            draw_list.add_rect(p1, p2, color)
                .filled(true)
                .build();
            
            // Label
            let label = format!("{} → {} ({:.2}ms)", 
                entry.from_phase, 
                entry.to_phase, 
                duration_ms
            );
            
            ui.set_cursor_screen_pos([cursor_pos[0] + 5.0, cursor_pos[1] + 3.0]);
            ui.text_colored([1.0, 1.0, 1.0, 1.0], &label);
            
            // Move cursor for next item
            ui.set_cursor_screen_pos([cursor_pos[0], cursor_pos[1] + bar_height + 5.0]);
        }
    }

    /// Render a heatmap of transition frequencies and times
    pub fn render_heatmap(ui: &Ui, tracker: &TrackedModule) {
        let stats = tracker.get_transition_statistics();
        
        if stats.is_empty() {
            ui.text("No transition data available.");
            return;
        }

        ui.text("Transition Frequency Heatmap:");
        ui.separator();

        // Find max count for color scaling
        let max_count = stats.iter().map(|(_, _, _, count)| *count).max().unwrap_or(1);
        let max_duration = stats.iter()
            .map(|(_, _, duration, _)| duration.as_micros())
            .max()
            .unwrap_or(1) as f32;

        if let Some(_table) = ui.begin_table_with_flags(
            "heatmap_table",
            5,
            imgui::TableFlags::BORDERS | imgui::TableFlags::ROW_BG,
        ) {
            ui.table_setup_column("From");
            ui.table_setup_column("To");
            ui.table_setup_column("Avg Time");
            ui.table_setup_column("Count");
            ui.table_setup_column("Heat");
            ui.table_headers_row();

            for (from, to, avg_duration, count) in stats {
                ui.table_next_row();
                
                ui.table_next_column();
                ui.text(format!("{}", from));
                
                ui.table_next_column();
                ui.text(format!("{}", to));
                
                ui.table_next_column();
                if avg_duration.as_millis() < 1 {
                    ui.text(format!("{:.2}μs", avg_duration.as_micros()));
                } else {
                    ui.text(format!("{:.2}ms", avg_duration.as_millis()));
                }
                
                ui.table_next_column();
                ui.text(format!("{}", count));
                
                ui.table_next_column();
                // Heat bar based on frequency
                let heat = count as f32 / max_count as f32;
                let color = Self::heat_color(heat);
                
                let bar_width = 100.0 * heat;
                imgui::ProgressBar::new(heat)
                    .size([bar_width, 0.0])
                    .build(ui);
            }
        }
    }

    /// Get color for a specific phase
    fn phase_color(phase: &HaCKLifecycleState) -> u32 {
        match phase {
            HaCKLifecycleState::Uninitialized => Self::rgb_to_u32(128, 128, 128),
            HaCKLifecycleState::Initializing => Self::rgb_to_u32(100, 150, 255),
            HaCKLifecycleState::Ready => Self::rgb_to_u32(100, 255, 100),
            HaCKLifecycleState::Updating => Self::rgb_to_u32(255, 200, 50),
            HaCKLifecycleState::PostUpdate => Self::rgb_to_u32(200, 150, 50),
            HaCKLifecycleState::RenderingMenu => Self::rgb_to_u32(150, 100, 255),
            HaCKLifecycleState::PostRenderMenu => Self::rgb_to_u32(120, 80, 200),
            HaCKLifecycleState::RenderingWindow => Self::rgb_to_u32(255, 100, 150),
            HaCKLifecycleState::PostRenderWindow => Self::rgb_to_u32(200, 80, 120),
            HaCKLifecycleState::RenderingDraw => Self::rgb_to_u32(100, 255, 255),
            HaCKLifecycleState::PostRenderDraw => Self::rgb_to_u32(80, 200, 200),
            HaCKLifecycleState::Unloading => Self::rgb_to_u32(255, 100, 100),
            HaCKLifecycleState::Error => Self::rgb_to_u32(255, 0, 0),
            HaCKLifecycleState::Qued => Self::rgb_to_u32(180, 180, 100),
            HaCKLifecycleState::Stasis => Self::rgb_to_u32(100, 100, 100),
        }
    }

    /// Get heat color (cool blue to hot red)
    fn heat_color(heat: f32) -> u32 {
        let heat = heat.clamp(0.0, 1.0);
        
        if heat < 0.5 {
            // Blue to yellow
            let t = heat * 2.0;
            let r = (t * 255.0) as u8;
            let g = (t * 255.0) as u8;
            let b = ((1.0 - t) * 255.0) as u8;
            Self::rgb_to_u32(r, g, b)
        } else {
            // Yellow to red
            let t = (heat - 0.5) * 2.0;
            let r = 255;
            let g = ((1.0 - t) * 255.0) as u8;
            let b = 0;
            Self::rgb_to_u32(r, g, b)
        }
    }

    /// Convert RGB to imgui color u32
    fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
        let a = 255u8;
        ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
    }

    /// Render compact overview showing time between key phases
    pub fn render_update_cycle_overview(ui: &Ui, tracker: &TrackedModule) {
        ui.text("Update Cycle Timings:");
        ui.separator();
        
        // Key transitions to monitor
        let key_transitions = vec![
            ("Idle → Update", HaCKLifecycleState::Stasis, HaCKLifecycleState::Updating),
            ("Update Duration",HaCKLifecycleState::Updating, HaCKLifecycleState::PostUpdate),
            ("Update → Idle", HaCKLifecycleState::PostUpdate, HaCKLifecycleState::Stasis),
            ("Idle → Render Menu", HaCKLifecycleState::Stasis, HaCKLifecycleState::RenderingMenu),
            ("Menu Duration", HaCKLifecycleState::RenderingMenu, HaCKLifecycleState::PostRenderMenu),
            ("Idle → Render Window", HaCKLifecycleState::Stasis, HaCKLifecycleState::RenderingWindow),
            ("Window Duration", HaCKLifecycleState::RenderingWindow, HaCKLifecycleState::PostRenderWindow),
            ("Idle → Render Draw", HaCKLifecycleState::Stasis, HaCKLifecycleState::RenderingDraw),
            ("Draw Duration", HaCKLifecycleState::RenderingDraw, HaCKLifecycleState::PostRenderDraw),
        ];

        for (label, from, to) in key_transitions {
            if let Some(avg) = tracker.get_average_transition_time(from, to) {
                let time_str = if avg.as_millis() < 1 {
                    format!("{:.2}μs", avg.as_micros())
                } else if avg.as_millis() < 1000 {
                    format!("{:.2}ms", avg.as_millis())
                } else {
                    format!("{:.2}s", avg.as_secs_f32())
                };
                
                ui.text(format!("{}: {}", label, time_str));
            }
        }

        // Time since last full update cycle
        if let Some(duration) = tracker.time_since_last_phase(HaCKLifecycleState::Updating) {
            ui.separator();
            ui.text_colored(
                [1.0, 1.0, 0.0, 1.0],
                &format!("Time since last update: {:.2}s", duration.as_secs_f32())
            );
        }
    }
}