use std::any::TypeId;

use crate::{
    gui::{Color, TreeNodeFlags, UiBackend, Vec2, WinCondition, WindowOptions},
    HaCKS,
};

impl HaCKS {
    pub fn render_debug_window(&mut self, ui: &dyn UiBackend) {
        if !*self.show_debug_window.borrow() {
            return;
        }

        let scale = ui.current_font_size() / 14.0;
        let mut show_window = *self.show_debug_window.borrow();

        let options = WindowOptions::new()
            .with_size([600.0 * scale, 800.0 * scale], WinCondition::FirstUseEver)
            .with_position([100.0 * scale, 100.0 * scale], WinCondition::FirstUseEver);

        if let Some(_token) =
            ui.begin_window_simple("Module Debug Viewer", &mut show_window, &options)
        {
            ui.text_colored(Color::rgba(1.0, 1.0, 0.0, 1.0), "Module Inspector");
            ui.separator();

            // --- First pass: collect (name, type_id) without borrowing modules later ---
            let mut module_list: Vec<(String, TypeId)> = self
                .hacs
                .iter()
                .map(|(id, m_rc)| {
                    let m = m_rc.borrow();
                    (m.name().to_string(), *id)
                })
                .collect();

            module_list.sort_by(|a, b| a.0.cmp(&b.0));

            // --- Second pass: borrow modules one-by-one safely ---
            for (name, type_id) in module_list {
                let module_rc = self.hacs.get(&type_id).unwrap();
                let module = module_rc.borrow(); // immutable borrow

                if ui.collapsing_header(&name, TreeNodeFlags::EMPTY) {
                    ui.indent();

                    // --- Metadata tree ---
                    if let Some(_t) = ui.tree_node("Metadata") {
                        let metadata = module.metadata();

                        ui.text(&format!("Name: {}", metadata.name));
                        ui.text(&format!("Description: {}", metadata.description));
                        ui.text(&format!("Category: {}", metadata.category));
                        ui.text(&format!("Hotkey: {:?}", metadata.hotkeys));

                        ui.separator();

                        ui.text(&format!("Menu Weight: {}", metadata.menu_weight));
                        ui.text(&format!("Window Weight: {}", metadata.window_weight));
                        ui.text(&format!("Draw Weight: {}", metadata.draw_weight));
                        ui.text(&format!("Update Weight: {}", metadata.update_weight));

                        ui.separator();

                        ui.text(&format!("Visible in GUI: {}", metadata.visible_in_gui));
                        ui.text(&format!("Menu Enabled: {}", metadata.is_menu_enabled));
                        ui.text(&format!("Window Enabled: {}", metadata.is_window_enabled));
                        ui.text(&format!("Render Enabled: {}", metadata.is_render_enabled));
                        ui.text(&format!("Update Enabled: {}", metadata.is_update_enabled));

                        ui.text(&format!("Window Pos: {:?}", metadata.window_pos));
                        ui.text(&format!("Window Size: {:?}", metadata.window_size));
                    }

                    // --- JSON tree ---
                    if let Some(_t) = ui.tree_node("Module Data (JSON)") {
                        match module.to_json_data_only() {
                            Ok(json) => {
                                let json_pretty = serde_json::to_string_pretty(&json)
                                    .unwrap_or_else(|_| "Failed to format JSON".to_string());

                                // Note: child_window is not in abstraction, using text_wrapped instead
                                ui.text_wrapped(&json_pretty);
                            }
                            Err(e) => {
                                ui.text_colored(
                                    Color::rgba(1.0, 0.0, 0.0, 1.0),
                                    &format!("Serialization error: {}", e),
                                );
                            }
                        }
                    }

                    ui.text_colored(
                        Color::rgba(0.6, 0.6, 0.6, 1.0),
                        &format!("TypeId: {:?}", type_id),
                    );

                    ui.unindent();
                    ui.separator();
                }
            }
        }

        if !show_window {
            *self.show_debug_window.borrow_mut() = show_window;
        }
    }

    pub fn render_metadata_editor_windows(&mut self, ui: &dyn UiBackend) {
        let scale = ui.current_font_size() / 14.0;
        // Visualization window
        let mut show_viz = *self.metadata_window_viz.borrow();
        if show_viz {
            let options = WindowOptions::new()
                .with_position([80.0 * scale, 0.0], WinCondition::FirstUseEver)
                .with_always_auto_resize(true);

            if let Some(_token) =
                ui.begin_window_simple("Module Weight Visualization", &mut show_viz, &options)
            {
                self.render_weight_visualization_window(ui);
            }
        }
        if !show_viz {
            *self.metadata_window_viz.borrow_mut() = show_viz;
        }

        // Metadata editor window
        let mut show_metadata = *self.metadata_window.borrow();
        if show_metadata {
            let options = WindowOptions::new()
                .with_position([80.0 * scale, 400.0], WinCondition::FirstUseEver)
                .with_always_auto_resize(true);

            if let Some(_token) =
                ui.begin_window_simple("Module Metadata Editor", &mut show_metadata, &options)
            {
                self.render_metadata_editor_window_content(ui);
            }
        }
        if !show_metadata {
            *self.metadata_window.borrow_mut() = show_metadata;
        }
    }

    pub fn render_weight_visualization_window(&mut self, ui: &dyn UiBackend) {
        // ui.text("Module Weight Visualization");
        // ui.same_line();
        // let dock_label = if self.metadata_window_viz { "Dock" } else { "Undock" };
        // if ui.button(&format!("{}##viz_dock", dock_label)) {
        //     self.metadata_window_viz = !self.metadata_window_viz;
        // }
        // ui.separator();

        self.render_weight_visualization_content(ui);
    }

    pub fn render_metadata_editor_window_content(&mut self, ui: &dyn UiBackend) {
        // --- Window Manager ---
        if ui.collapsing_header("Window Manager", TreeNodeFlags::EMPTY) {
            ui.indent();
            ui.text_colored(Color::rgba(0.7, 0.7, 1.0, 1.0), "Undocked Groups:");

            let mut to_remove = Vec::new();
            for (path, is_open) in self.windowed_groups.borrow_mut().iter() {
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
                self.windowed_groups.borrow_mut().insert(path, false);
            }

            if self.windowed_groups.borrow().is_empty()
                || self.windowed_groups.borrow().values().all(|v| !v)
            {
                ui.text_disabled("(no undocked groups)");
            }

            ui.unindent();
            ui.separator();
        }

        // --- Legend ---
        if ui.collapsing_header("Legend", TreeNodeFlags::EMPTY) {
            ui.text_colored(
                Color::from([0.5, 0.5, 1.0, 1.0]),
                "Higher weight = runs/renders first",
            );
            ui.text_colored(
                Color::from([1.0, 1.0, 0.5, 1.0]),
                "Update enabled = module.update() runs",
            );
            ui.text_colored(
                Color::from([1.0, 0.5, 1.0, 1.0]),
                "Render enabled = module.render_draw() runs",
            );
            ui.separator();
        }

        // --- Collect module list (immutable borrow) ---
        let mut module_list: Vec<(String, TypeId)> = self
            .hacs
            .iter()
            .map(|(id, m_rc)| {
                let m = m_rc.borrow();
                (m.name().to_string(), *id)
            })
            .collect();
        module_list.sort_by(|a, b| a.0.cmp(&b.0));

        // --- Iterate modules ---
        for (idx, (name, type_id)) in module_list.iter().enumerate() {
            // --- Mutable borrow block ---
            {
                let module_rc = self.hacs.get(type_id).unwrap();
                let mut module = module_rc.borrow_mut();
                let id_token = ui.push_id_usize(idx);

                if ui.collapsing_header(&name, TreeNodeFlags::EMPTY) {
                    ui.indent();
                    ui.columns(2, "metadata", true);
                    ui.set_column_width(0, 150.0);

                    // Menu Weight
                    ui.text("Menu Weight:");
                    ui.next_column();
                    let mut menu_weight = module.menu_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float_with_step("##mw", &mut menu_weight, 0.1, 1.0) {
                        module.set_menu_weight(menu_weight.max(0.0));
                    }
                    ui.next_column();

                    // Window Weight
                    ui.text("Window Weight:");
                    ui.next_column();
                    let mut window_weight = module.window_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float_with_step("##rw", &mut window_weight, 0.1, 1.0) {
                        module.set_window_weight(window_weight.max(0.0));
                    }
                    ui.next_column();

                    // Draw Weight
                    ui.text("Draw Weight:");
                    ui.next_column();
                    let mut draw_weight = module.draw_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float_with_step("##dw", &mut draw_weight, 0.1, 1.0) {
                        module.set_draw_weight(draw_weight.max(0.0));
                    }
                    ui.next_column();

                    // Update Weight
                    ui.text("Update Weight:");
                    ui.next_column();
                    let mut update_weight = module.update_weight();
                    ui.set_next_item_width(-1.0);
                    if ui.input_float_with_step("##uw", &mut update_weight, 0.1, 1.0) {
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

                    ui.text("Window Position:");
                    ui.next_column();
                    ui.set_next_item_width(-1.0);
                    ui.input_float2("##wpos", &mut module.metadata_mut().window_pos);
                    ui.next_column();

                    ui.text("Window Size:");
                    ui.next_column();
                    ui.set_next_item_width(-1.0);
                    ui.input_float2("##wsize", &mut module.metadata_mut().window_size);
                    ui.next_column();

                    // Auto Resize Window
                    ui.text("Auto Resize:");
                    ui.next_column();
                    let mut auto_resize = module.metadata().auto_resize_window;
                    if ui.checkbox("##autoresize", &mut auto_resize) {
                        module.metadata_mut().auto_resize_window = auto_resize;
                    }
                    ui.next_column();

                    ui.columns(1, "", false);

                    // Hotkeys
                    if ui.collapsing_header("Hotkeys", TreeNodeFlags::EMPTY) {
                        ui.indent();
                        let hotkeys_modified =
                            module.metadata_mut().render_hotkey_config_simple(ui);
                        if hotkeys_modified {
                            self.hotkey_manager
                                .borrow_mut()
                                .sync_from_bindings(*type_id, &module.metadata().hotkeys);
                        }
                        ui.unindent();
                    }

                    ui.unindent();
                }

                // No need to call pop() - RAII handles it
            } // <- mutable borrow ends here

            // --- Immutable borrow block for dependencies ---
            {
                let deps = self
                    .hacs
                    .get(type_id)
                    .unwrap()
                    .borrow()
                    .update_dependencies();
                if !deps.is_empty() {
                    ui.text_colored(Color::from([0.7, 0.7, 0.7, 1.0]), "Dependencies:");
                    ui.same_line();
                    let dep_names: Vec<String> = deps
                        .iter()
                        .filter_map(|id| {
                            self.hacs
                                .get(id)
                                .map(|m_rc| m_rc.borrow().name().to_string())
                        })
                        .collect();
                    ui.text(&dep_names.join(", "));
                }
            }
        }
    }

    pub fn render_weight_visualization_content(&mut self, ui: &dyn UiBackend) {
        if ui.radio_button("Menu Order", *self.viz_mode.borrow() == 0) {
            *self.viz_mode.borrow_mut() = 0;
        }
        ui.same_line();
        if ui.radio_button("Window Order", *self.viz_mode.borrow() == 1) {
            *self.viz_mode.borrow_mut() = 1;
        }
        ui.same_line();
        if ui.radio_button("Draw Order", *self.viz_mode.borrow() == 2) {
            *self.viz_mode.borrow_mut() = 2;
        }
        ui.same_line();
        if ui.radio_button("Update Order", *self.viz_mode.borrow() == 3) {
            *self.viz_mode.borrow_mut() = 3;
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
        if let Some(_cb) = ui.begin_combo(
            "##color_scheme",
            &format!(
                "Color Scheme: {}",
                color_schemes[*self.color_scheme.borrow()]
            ),
        ) {
            for (idx, scheme) in color_schemes.iter().enumerate() {
                if *self.color_scheme.borrow() == idx {
                    ui.set_item_default_focus();
                }
                let clicked = ui
                    .selectable(scheme)
                    .selected(*self.color_scheme.borrow() == idx)
                    .build();
                if clicked {
                    *self.color_scheme.borrow_mut() = idx;
                }
            }
        }

        ui.separator();

        let mut sorted_modules: Vec<(String, f32, TypeId)> = Vec::new();

        for (id, module_rc) in self.hacs.iter() {
            let module = module_rc.borrow(); // immutable borrow for reading weights
            let (weight, enabled) = match *self.viz_mode.borrow() {
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

        // Optionally, sort descending by weight
        sorted_modules.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        sorted_modules.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let draw_list = ui.get_window_draw_list();
        let mut draw_list = draw_list;
        let x = ui.get_cursor_screen_pos().x;
        let available_width = ui.get_content_region_avail().x;
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

        let mut current_y = ui.get_cursor_screen_pos().y;
        let mut order = 1;
        let active_count = sorted_modules.iter().filter(|(_, w, _)| *w >= 0.0).count();

        for (name, weight, _) in sorted_modules.iter() {
            if *weight < 0.0 {
                continue;
            }

            let bar_width = (weight * scale).max(5.0);
            let progress = order as f32 / active_count.max(1) as f32;
            let bar_color = self.get_bar_color(order, active_count, progress);

            draw_list.add_rect(
                Vec2::new(x + 80.0, current_y),
                Vec2::new(x + 80.0 + bar_width, current_y + bar_height),
                Color::rgba(bar_color[0], bar_color[1], bar_color[2], bar_color[3]),
                true,
            );

            draw_list.add_rect(
                Vec2::new(x + 80.0, current_y),
                Vec2::new(x + 80.0 + bar_width, current_y + bar_height),
                Color::rgba(200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 1.0),
                true,
            );

            draw_list.add_text(
                Vec2::new(x + 5.0, current_y + bar_height / 2.0 - 6.0),
                Color::rgba(1.0, 1.0, 1.0, 1.0),
                &format!("{:>2}", order),
            );

            draw_list.add_text(
                Vec2::new(x + 85.0, current_y + bar_height / 2.0 - 6.0),
                Color::rgba(1.0, 1.0, 1.0, 1.0),
                &format!("{}: {:.2}", name, weight),
            );

            current_y += bar_height + spacing;
            order += 1;
        }

        let total_height = (order as f32 - 1.0) * (bar_height + spacing);
        ui.dummy(Vec2::new(0.0, total_height));
    }

    pub fn render_window_manager(&mut self, ui: &dyn UiBackend) {
        // --- Manage Menus ---
        if ui.collapsing_header("Manage menus", TreeNodeFlags::EMPTY) {
            if let Some(cache) = &*self.menu_cache.borrow_mut() {
                ui.text("Menu Groups: Check to undock grouped menus/sub menus");

                for (top_name, _) in cache.top_level.iter() {
                    let path = vec![top_name.clone()];
                    let is_windowed = self
                        .windowed_groups
                        .borrow()
                        .get(&path)
                        .copied()
                        .unwrap_or(false);

                    let mut checked = is_windowed;
                    if ui.checkbox(&top_name, &mut checked) {
                        self.windowed_groups.borrow_mut().insert(path, checked);
                    }
                }
            }
        }

        // --- Module Windows ---
        if ui.collapsing_header("Module Windows", TreeNodeFlags::EMPTY) {
            ui.text("Modules: Check to undock");

            // 1. Immutable borrow: collect names + TypeIds
            let mut module_list: Vec<(String, TypeId)> = self
                .hacs
                .iter()
                .map(|(id, m_rc)| {
                    let m = m_rc.borrow();
                    (m.name().to_string(), *id)
                })
                .collect();

            module_list.sort_by(|a, b| a.0.cmp(&b.0));

            // 2. Now it's safe to mut-borrow individual entries
            for (name, type_id) in module_list {
                if let Some(module_rc) = self.hacs.get(&type_id) {
                    let mut module = module_rc.borrow_mut();

                    let mut checked = module.is_window_enabled();
                    if ui.checkbox(&name, &mut checked) {
                        module.set_show_window(checked);
                    }
                }
            }
        }
    }
}
