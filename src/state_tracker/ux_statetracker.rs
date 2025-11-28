
use std::time::Duration;
use imgui::{TableFlags, Ui};
use crate::state_tracker::statetracker::*;

pub trait StateStatsRenderer {
    fn render_state_stats<T: Clone + PartialEq, F: StateFormatter<T>>(
        &self,
        ui: &Ui,
        stats: &[(T, Duration, u32, Option<Duration>)],
        total_time: Duration,
        formatter: &F
    );
}

/// Renders state stats with progress bars embedded in state cells
pub struct ProgressBarStateRenderer;


impl StateStatsRenderer for ProgressBarStateRenderer {
    fn render_state_stats<T: Clone + PartialEq, F: StateFormatter<T>>(
        &self,
        ui: &Ui,
        stats: &[(T, Duration, u32, Option<Duration>)],
        total_time: Duration,
        formatter: &F
    ) {
        if let Some(_table) = ui.begin_table_with_flags(
            "state_stats_table", 
            5, 
            TableFlags::BORDERS | TableFlags::ROW_BG | TableFlags::RESIZABLE | TableFlags::SIZING_FIXED_FIT
        ) {
            let mut max_state_width = 0.0;
            let mut max_time_width = 0.0;
            let mut max_count_width = 0.0;
            let mut max_avg_width = 0.0;
            for (state, time, count, avg_time) in stats {
                // Calculate the width of the state text
                let state_str = formatter.format_state(state);
                let text_size = ui.calc_text_size(&state_str);
                let text_width = text_size[0] + 5.0; // Add padding
                if text_width > max_state_width {
                    max_state_width = text_width;
                }
                // Calculate the width of the time text
                let time_str = if time.as_millis() < 1 {
                    format!("{:.2}μs", time.as_micros())
                } else {
                    format!("{:.2}ms", time.as_millis())
                };
                let time_text_size = ui.calc_text_size(&time_str);
                let time_text_width = time_text_size[0] + 5.0; // Add padding
                if time_text_width > max_time_width {
                    max_time_width = time_text_width;
                }
                // Calculate the width of the count text
                let count_str = format!("{}", count);
                let count_text_size = ui.calc_text_size(&count_str);    
                let count_text_width = count_text_size[0] + 5.0; // Add padding
                if count_text_width > max_count_width {
                    max_count_width = count_text_width;
                }
                // Calculate the width of the average text
                let avg_str = if let Some(avg) = avg_time {
                    if avg.as_millis() < 1 {
                        format!("{:.2}μs", avg.as_micros())
                    } else {
                        format!("{:.2}ms", avg.as_millis())
                    }
                } else {
                    "-".to_string()
                };
                let avg_text_size = ui.calc_text_size(&avg_str);
                let avg_text_width = avg_text_size[0] + 5.0; // Add padding 
                if avg_text_width > max_avg_width {
                    max_avg_width = avg_text_width;
                }
            }
            ui.set_next_item_width(max_state_width);
            ui.table_setup_column("State");
            ui.set_next_item_width(max_state_width);
            ui.table_setup_column("percent");
            ui.set_next_item_width(max_count_width);
            ui.table_setup_column("Count");
            ui.set_next_item_width(max_time_width);
            ui.table_setup_column("Total");
            ui.set_next_item_width(max_avg_width);
            ui.table_setup_column("Avg");
            ui.table_headers_row();
            for (state, time, count, avg_time) in stats {
                if time.is_zero() { continue; }
                ui.table_next_row();
                // State column with embedded progress bar
                ui.table_next_column();
                ui.text(formatter.format_state(state));
                //percentage
                ui.table_next_column();
                let color1 = [0.0, 0.0, 0.0, 1.0]; // Black
                let color2 = [1.0, 1.0, 1.0, 1.0]; // White
                self.render_state_cell(ui, state, *time, total_time, formatter,color1, color2);
                // Count
                ui.table_next_column();
                ui.text(format!("{}", count));
                // Total time
                ui.table_next_column();
                ui.text(format!("{:.2}s", time.as_secs_f32()));
                // Average
                ui.table_next_column();
                if let Some(avg) = avg_time {
                    if avg.as_millis() < 1 {
                        ui.text(format!("{:.2}μs", avg.as_micros()));
                    } else {
                        ui.text(format!("{:.2}ms", avg.as_millis()));
                    }
                } else {
                    ui.text("-");
                }
            }
        }
    }
}

impl ProgressBarStateRenderer {
    /// Renders a state cell with progress bar and dual-colored text
    fn render_state_cell<T: Clone + PartialEq, F: StateFormatter<T>>(
        &self,
        ui: &Ui,
        state: &T,
        time: Duration,
        total_time: Duration,
        formatter: &F,
        _color1: [f32; 4],
        color2: [f32; 4],
    ) {
        let start_pos = ui.cursor_pos();
        let cell_width = ui.current_column_width();
        // Get state text
        let state_str = formatter.format_state(state);
        let text_size = ui.calc_text_size(&state_str);
        let text_height = text_size[1];
        let bar_height = text_height + 2.0;
        // Calculate percentage
        let percentage = if total_time.is_zero() { 
            0.0 
        } else { 
            (time.as_secs_f64() / total_time.as_secs_f64()) * 100.0 
        };
        // Draw progress bar (no text)
        imgui::ProgressBar::new(percentage as f32 / 100.0)
            .size([cell_width - 10.0, bar_height])
            .build(ui);
        // Text positioning (centered vertically in bar)
        let text_y = start_pos[1] + (bar_height - text_height) * 0.5;
        let text_x = start_pos[0] + 5.0;
        // Background text (dimmed color)
        ui.set_cursor_pos([text_x, text_y]);
        // ui.text_colored(color1, &state_str);
        // Foreground text (bright color, clipped to progress width)
        let clip_width = (cell_width - 10.0) * (percentage as f32 / 100.0);
        let draw_list = ui.get_window_draw_list();
        draw_list.with_clip_rect(
            [text_x, start_pos[1]], 
            [text_x + clip_width, start_pos[1] + bar_height],
            || {
                ui.set_cursor_pos([text_x, text_y]);
                ui.text_colored(color2, &state_str);
            }
        );
    }
}

/// Extension trait for rendering state tracker stats in UI
pub trait RenderableStateTracker {
    fn render_stats<R: StateStatsRenderer>(&self, ui: &Ui, renderer: &R);
}

impl<T: Clone + PartialEq, F: StateFormatter<T>> RenderableStateTracker for StateTracker<T, F> {
    fn render_stats<R: StateStatsRenderer>(&self, ui: &Ui, renderer: &R) {
        let stats = self.get_stats();
        let total_time = self.get_active_time();
        renderer.render_state_stats(ui, &stats, total_time, &self.formatter);
    }
}

pub trait StateTrackerUI {
    fn render_stats_tracker_ui(&mut self, ui: &Ui);
}

impl<T: Clone + PartialEq, F: StateFormatter<T>> StateTrackerUI for StateTracker<T, F> {
    fn render_stats_tracker_ui(&mut self, ui: &Ui) {
        let mut show = self.show;
        ui.window("State Tracker")
                .opened(&mut show)
                .resizable(true)
                .size([300.0, 400.0], imgui::Condition::FirstUseEver)
                .always_auto_resize(true)
                .menu_bar(true)
                .build(|| {
            // Menu bar
            if let Some(_menu_bar) = ui.begin_menu_bar() {
                ui.menu("Options", || {
                    if ui.menu_item("Reset Stats") {
                        self.reset();
                    }
                    if ui.menu_item("Close") {
                        self.hide();
                    }
                });
            }
            // Basic stats
       
            ui.text(format!("Session: {:.2}s (Active: {:.2}s)", 
                self.get_total_session_duration().as_secs_f32(),
                self.get_active_time().as_secs_f32())
            );
            ui.separator();
            // Current state
            if let Some(ref state) = self.current_state {
                if let Some(start) = self.current_state_start {
                    ui.text(format!(
                        "Current: {} ({:.2}s)",
                        self.formatter.format_state(state),
                        start.elapsed().as_secs_f32()
                    ));
                }
                ui.separator();
            }
            let ProgressBarStateRenderer = ProgressBarStateRenderer;
            self.render_stats( ui, &ProgressBarStateRenderer);
        });
        if show == false {
            self.hide();
        }
    }
}