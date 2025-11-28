#[cfg(feature = "gui")]
pub mod gui_integration {
    use imgui::Ui;
    use crate::{GlobalStateTracker, ModuleLifecycleState, StateViewMode, state_tracker::ux_statetracker::{
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
                    
                    ui.text(format!(
                        "Active Session: {:.2}s",
                        tracker.lifecycle_tracker.get_active_session_duration().as_secs_f32()
                    ));
                    
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
            let mut lifecycle_stats: HashMap<ModuleLifecycleState, (Duration, u64)> = HashMap::new();

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
    }
}