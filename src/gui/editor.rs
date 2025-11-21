use std::any::TypeId;

use imgui::{Condition, TreeNodeFlags, Ui};

use crate::HaCKS;

impl HaCKS {
    pub fn render_debug_window(&mut self, ui: &Ui) {
        if !self.show_debug_window {
            return;
        }
        let scale = ui.current_font_size()/14.0;
        ui.window("Module Debug Viewer")
            .opened(&mut self.show_debug_window)
            .size([600.0 * scale, 800.0 * scale], Condition::FirstUseEver)
            .position([100.0 * scale, 100.0 * scale], Condition::FirstUseEver)
            .build(|| {
                ui.text_colored([1.0, 1.0, 0.0, 1.0], "Module Inspector");
                ui.separator();
                
                // Sort modules by name for easier browsing
                let mut modules: Vec<_> = self.hacs.iter().collect();
                modules.sort_by(|a, b| {
                    a.1.name().cmp(&b.1.name())
                });
                
                for (type_id, module) in modules {
                    let name = module.name().to_string();
                    
                    if ui.collapsing_header(&name, imgui::TreeNodeFlags::empty()) {
                        ui.indent();
                        
                        // Metadata section
                        if let Some(_token) = ui.tree_node("Metadata") {
                            let metadata = module.metadata();
                            ui.text(format!("Name: {}", metadata.name));
                            ui.text(format!("Description: {}", metadata.description));
                            ui.text(format!("Category: {}", metadata.category));
                            ui.text(format!("Hotkey: {:?}", metadata.hotkeys));
                            ui.separator();
                            ui.text(format!("Menu Weight: {}", metadata.menu_weight));
                            ui.text(format!("Window Weight: {}", metadata.window_weight));
                            ui.text(format!("Draw Weight: {}", metadata.draw_weight));
                            ui.text(format!("Update Weight: {}", metadata.update_weight));
                            ui.separator();
                            ui.text(format!("Visible in GUI: {}", metadata.visible_in_gui));
                            ui.text(format!("Menu Enabled: {}", metadata.is_menu_enabled));
                            ui.text(format!("Window Enabled: {}", metadata.is_window_enabled));
                            ui.text(format!("Render Enabled: {}", metadata.is_render_enabled));
                            ui.text(format!("Update Enabled: {}", metadata.is_update_enabled));
                            ui.text(format!("Window Pos: {:?}", metadata.window_pos));
                            ui.text(format!("Window Size: {:?}", metadata.window_size));
                        }
                        
                        // Serialize the entire module as JSON
                        if let Some(_token) = ui.tree_node("Module Data (JSON)") {
                            match module.to_json_data_only() {
                                Ok(json) => {
                                    let json_str = serde_json::to_string_pretty(&json)
                                        .unwrap_or_else(|_| "Failed to format JSON".to_string());
                                    
                                    // Display in a scrollable child window
                                    ui.child_window("##json_scroll")
                                        .size([550.0 * scale, 400.0 * scale])
                                        .build(|| {
                                            ui.text_wrapped(&json_str);
                                        });
                                }
                                Err(e) => {
                                    ui.text_colored([1.0, 0.0, 0.0, 1.0], format!("Serialization error: {}", e));
                                }
                            }
                        }
                        
                        // TypeId for reference
                        ui.text_colored([0.5, 0.5, 0.5, 1.0], format!("TypeId: {:?}", type_id));
                        
                        ui.unindent();
                        ui.separator();
                    }
                }
            });
    }
    
    pub fn render_metadata_editor_windows(&mut self, ui: &imgui::Ui) {
        let scale = ui.current_font_size()/14.0;
        // Visualization window
        let mut show_viz = self.metadata_window_viz;
        if show_viz {
            ui.window("Module Weight Visualization")
                .opened(&mut show_viz)
                .always_auto_resize(true)
                .position([80.0 * scale, 0.0], Condition::FirstUseEver)
                .build(|| {
                    self.render_weight_visualization_window(ui);
                });
        }
        if !show_viz {
            self.metadata_window_viz = show_viz;
        }
    
        // Metadata editor window
        let mut show_metadata = self.metadata_window;
        if show_metadata {
            ui.window("Module Metadata Editor")
                .opened(&mut show_metadata)
                .always_auto_resize(true)
                .position([80.0 * scale, 400.0], Condition::FirstUseEver)
                .build(|| {
                    self.render_metadata_editor_window_content(ui);
                });
        }
        if !show_metadata {
            self.metadata_window = show_metadata;
        }
    }
    
    pub fn render_weight_visualization_window(&mut self, ui: &imgui::Ui) {
        // ui.text("Module Weight Visualization");
        // ui.same_line();
        // let dock_label = if self.metadata_window_viz { "Dock" } else { "Undock" };
        // if ui.button(&format!("{}##viz_dock", dock_label)) {
        //     self.metadata_window_viz = !self.metadata_window_viz;
        // }
        // ui.separator();
        
        self.render_weight_visualization_content(ui);
    }
        
    pub fn render_metadata_editor_window_content(&mut self, ui: &imgui::Ui) {
        if ui.collapsing_header("Window Manager", imgui::TreeNodeFlags::empty()) {
            ui.indent();
            ui.text_colored([0.7, 0.7, 1.0, 1.0], "Undocked Groups:");
            
            let mut to_remove = Vec::new();
            for (path, is_open) in self.windowed_groups.iter() {
                if *is_open {
                    let path_str = path.join(" > ");
                    ui.text(&path_str);
                    ui.same_line();
                    let dock_label = format!("Dock##dock_{}", path_str);
                    if ui.small_button(&dock_label) {
                        to_remove.push(path.clone());
                    }
                }
            }
            
            for path in to_remove {
                self.windowed_groups.insert(path, false);
            }
            
            if self.windowed_groups.is_empty() || self.windowed_groups.values().all(|v| !v) {
                ui.text_disabled("(no undocked groups)");
            }
            
            ui.unindent();
            ui.separator();
        }
    
        if ui.collapsing_header("Legend", imgui::TreeNodeFlags::empty()) {
            ui.text_colored([0.5, 0.5, 1.0, 1.0], "Higher weight = runs/renders first");
            ui.text_colored([1.0, 1.0, 0.5, 1.0], "Update enabled = module.update() runs");
            ui.text_colored([1.0, 0.5, 1.0, 1.0], "Render enabled = module.render_draw() runs");
            ui.separator();
        }
        
        let mut module_list: Vec<(String, TypeId)> = self.hacs
            .iter()
            .map(|(id, m)| (m.name().to_string(), *id))
            .collect();
        module_list.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (idx, (name, type_id)) in module_list.iter().enumerate() {
            if let Some(module) = self.hacs.get_mut(type_id) {
                let id_token = ui.push_id_usize(idx);
                
                if ui.collapsing_header(&name, imgui::TreeNodeFlags::empty()) {
                    ui.indent();
                    
                    ui.columns(2, "metadata", true);
                    ui.set_column_width(0, 150.0);
                    
                    // Menu Weight
                    ui.text("Menu Weight:");
                    ui.next_column();
                    let mut menu_weight = module.menu_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float("##mw", &mut menu_weight)
                        .step(0.1)
                        .step_fast(1.0)
                        .build() 
                    {
                        module.set_menu_weight(menu_weight.max(0.0));
                    }
                    ui.next_column();
                    
                    // Window Weight
                    ui.text("Window Weight:");
                    ui.next_column();
                    let mut weight_render = module.window_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float("##rw", &mut weight_render)
                        .step(0.1)
                        .step_fast(1.0)
                        .build() 
                    {
                        module.set_window_weight(weight_render.max(0.0));
                    }
                    ui.next_column();
    
                    // Draw Weight
                    ui.text("Draw Weight:");
                    ui.next_column();
                    let mut draw_weight = module.draw_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float("##dw", &mut draw_weight)
                        .step(0.1)
                        .step_fast(1.0)
                        .build() 
                    {
                        module.set_draw_weight(draw_weight.max(0.0));
                    }
                    ui.next_column();
                    
                    // Update Weight
                    ui.text("Update Weight:");
                    ui.next_column();
                    let mut update_weight = module.update_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float("##uw", &mut update_weight)
                        .step(0.1)
                        .step_fast(1.0)
                        .build() 
                    {
                        module.set_update_weight(update_weight.max(0.0));
                    }
                    ui.next_column();
                    
                    // Update Enabled
                    ui.text("Update Enabled:");
                    ui.next_column();
                    let mut update_enabled = module.is_update_enabled();
                    if ui.checkbox("##ue", &mut update_enabled) {
                        module.set_update_enabled(update_enabled);
                    }
                    ui.next_column();
                    
                    // Render Enabled
                    ui.text("Render Enabled:");
                    ui.next_column();
                    let mut render_enabled = module.is_render_enabled();
                    if ui.checkbox("##re", &mut render_enabled) {
                        module.set_render_enabled(render_enabled);
                    }
                    ui.next_column();
                    
                    // Menu Enabled
                    ui.text("Menu Enabled:");
                    ui.next_column();
                    let mut menu_enabled = module.is_menu_enabled();
                    if ui.checkbox("##me", &mut menu_enabled) {
                        module.set_show_menu(menu_enabled);
                    }
                    ui.next_column();
                    
                    // Window Enabled
                    ui.text("Window Enabled:");
                    ui.next_column();
                    let mut window_enabled = module.is_window_enabled();
                    if ui.checkbox("##we", &mut window_enabled) {
                        module.set_show_window(window_enabled);
                    }
                    ui.next_column();
                    
                    ui.columns(1, "", false);
                    
                    let deps = module.update_dependencies();
                    if !deps.is_empty() {
                        ui.text_colored([0.7, 0.7, 0.7, 1.0], "Dependencies:");
                        ui.same_line();
                        let dep_names: Vec<String> = deps.iter()
                            .filter_map(|id| self.hacs.get(id).map(|m| m.name().to_string()))
                            .collect();
                        ui.text(dep_names.join(", "));
                    }
                    
                    ui.unindent();
                }
                
                id_token.pop();
            }
        }
    }
    
    pub fn render_weight_visualization_content(&mut self, ui: &Ui) {
        if ui.radio_button_bool("Menu Order", self.viz_mode == 0) {
            self.viz_mode = 0;
        }
        ui.same_line();
        if ui.radio_button_bool("Window Order", self.viz_mode == 1) {
            self.viz_mode = 1;
        }
        ui.same_line();
        if ui.radio_button_bool("Draw Order", self.viz_mode == 2) {
            self.viz_mode = 2;
        }
        ui.same_line();
        if ui.radio_button_bool("Update Order", self.viz_mode == 3) {
            self.viz_mode = 3;
        }
        
        ui.separator();
        
        let color_schemes = vec![
            "Warm-Cool",
            "Blue-Cyan",
            "Muted",
            "Sunset",
            "Forest",
            "Neon",
            "Pastel",
            "Grayscale",
            "Monochrome Blue",
            "Rainbow (Legacy)",
        ];
        if let Some(_cb) = ui.begin_combo("##color_scheme", format!("Color Scheme: {}", color_schemes[self.color_scheme])) {
            for (idx, scheme) in color_schemes.iter().enumerate() {
                if self.color_scheme == idx {
                    ui.set_item_default_focus();
                }
                let clicked = ui.selectable_config(scheme)
                    .selected(self.color_scheme == idx)
                    .build();
                if clicked {
                    self.color_scheme = idx;
                }
            }
        }
        
        ui.separator();
        
        let mut sorted_modules = Vec::new();
        
        for (id, module) in self.hacs.iter() {
            let (weight, enabled) = match self.viz_mode {
                0 => (module.menu_weight(), module.is_menu_enabled()),
                1 => (module.window_weight(), module.is_window_enabled()),
                2 => (module.draw_weight(), module.is_render_enabled()),
                3 => (module.update_weight(), module.is_update_enabled()),
                _ => (0.0, false),
            };
            
            sorted_modules.push((
                module.name().to_string(),
                if enabled { weight } else { -1.0 },
                *id,
            ));
        }
        
        sorted_modules.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let draw_list = ui.get_window_draw_list();
        let [x, _y] = ui.cursor_screen_pos();
        let available_width = ui.content_region_avail()[0];
        let bar_height = 24.0;
        let spacing = 5.0;
        
        let mut max_weight = 0.0;
        for (_, weight, _) in &sorted_modules {
            if *weight > max_weight {
                max_weight = *weight;
            }
        }
        
        if max_weight == 0.0 {
            max_weight = 1.0;
        }
        
        let scale = (available_width - 100.0) / max_weight;
        
        ui.text("Execution order (top to bottom):");
        ui.spacing();
        
        let mut current_y = ui.cursor_screen_pos()[1];
        let mut order = 1;
        let active_count = sorted_modules.iter().filter(|(_, w, _)| *w >= 0.0).count();
        
        for (name, weight, _) in sorted_modules.iter() {
            if *weight < 0.0 {
                continue;
            }
            
            let bar_width = (weight * scale).max(5.0);
            let progress = order as f32 / active_count.max(1) as f32;
            let bar_color = self.get_bar_color(order, active_count, progress);
            
            draw_list
                .add_rect(
                    [x + 80.0, current_y],
                    [x + 80.0 + bar_width, current_y + bar_height],
                    imgui::ImColor32::from_rgba(
                        (bar_color[0] * 255.0) as u8,
                        (bar_color[1] * 255.0) as u8,
                        (bar_color[2] * 255.0) as u8,
                        (bar_color[3] * 255.0) as u8,
                    ),
                )
                .filled(true)
                .build();
            
            draw_list
                .add_rect(
                    [x + 80.0, current_y],
                    [x + 80.0 + bar_width, current_y + bar_height],
                    imgui::ImColor32::from_rgba(200, 200, 200, 255),
                )
                .build();
            
            draw_list.add_text(
                [x + 5.0, current_y + bar_height / 2.0 - 6.0],
                imgui::ImColor32::from_rgba(255, 255, 255, 255),
                &format!("{:>2}", order),
            );
            
            draw_list.add_text(
                [x + 85.0, current_y + bar_height / 2.0 - 6.0],
                imgui::ImColor32::from_rgba(255, 255, 255, 255),
                &format!("{}: {:.2}", name, weight),
            );
            
            current_y += bar_height + spacing;
            order += 1;
        }
        
        let total_height = (order as f32 - 1.0) * (bar_height + spacing);
        ui.dummy([0.0, total_height]);
    }

    pub fn render_window_manager(&mut self, ui: &Ui) {
        if ui.collapsing_header("Manage menus", TreeNodeFlags::empty()){
            // Show all available menu groups
            if let Some(cache) = &self.menu_cache {
                ui.text("Menu Groups: Check to undock grouped menus/sub menus");
                for (top_name, _entries) in cache.top_level.iter() {
                    let path = vec![top_name.clone()];
                    let is_windowed = self.windowed_groups
                        .get(&path)
                        .copied()
                        .unwrap_or(false);
                    
                    let mut checked = is_windowed;
                    if ui.checkbox(&format!("{}", top_name), &mut checked) {
                        self.windowed_groups.insert(path, checked);
                    }
                }
            }
        }
        
        if ui.collapsing_header("Module Windows", TreeNodeFlags::empty()){
            // Individual module windows
            ui.text("Modules: Check to undock");
            let mut module_list: Vec<(String, std::any::TypeId)> = self.hacs
                .iter()
                .map(|(id, m)| (m.name().to_string(), *id))
                .collect();
            module_list.sort_by(|a, b| a.0.cmp(&b.0));
            
            for (name, type_id) in module_list {
                if let Some(module) = self.hacs.get_mut(&type_id) {
                    let mut checked = module.is_window_enabled();
                    if ui.checkbox(&format!("{}", name), &mut checked) {
                        module.set_show_window(checked);
                    }
                }
            }
        }
    }

}