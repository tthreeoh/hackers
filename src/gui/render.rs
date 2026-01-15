use crate::{
    gui::{DrawList, UiBackend, WinCondition, WindowOptions},
    HaCK, HaCKS,
};
use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
};

impl HaCKS {
    fn render_group_window(
        &mut self,
        ui: &dyn UiBackend,
        name: &str,
        entries: &[(Vec<String>, String)],
        path: &[String],
    ) {
        let mut show = true;
        let scale = ui.current_font_size() / 14.0;

        let options = WindowOptions::new()
            .with_position([80.0 * scale, 100.0], WinCondition::FirstUseEver)
            .with_always_auto_resize(true);

        if let Some(_token) = ui.begin_window_simple(name, &mut show, &options) {
            self.render_grouped_entries_in_window(ui, entries, 1, path);
        }

        if !show {
            self.windowed_groups.borrow_mut().remove(path);
        }
    }

    fn render_grouped_entries_in_window(
        &mut self,
        ui: &dyn UiBackend,
        entries: &[(Vec<String>, String)],
        depth: usize,
        current_path: &[String],
    ) {
        let mut groups: BTreeMap<Option<String>, Vec<(Vec<String>, String)>> = BTreeMap::new();

        for (path, name_key) in entries {
            if depth >= path.len() {
                groups
                    .entry(None)
                    .or_default()
                    .push((path.clone(), name_key.clone()));
            } else {
                let submenu_name = path[depth].clone();
                groups
                    .entry(Some(submenu_name))
                    .or_default()
                    .push((path.clone(), name_key.clone()));
            }
        }

        // Render modules that end at this level
        if let Some(terminal_entries) = groups.remove(&None) {
            for (_path, name_key) in terminal_entries {
                if let Some(module_rc) = self.hacs.get(&name_key) {
                    // Mutable borrow for actions
                    let mut module = module_rc.borrow_mut();
                    if module.is_menu_enabled() && !module.is_window_enabled() {
                        let name = module.name().to_string();
                        let type_id = module.nac_type_id();

                        if let Some(_menu) = ui.begin_menu(&name) {
                            module.render_menu(ui); // render_menu can be called on mutable borrow if it mutates state
                        }

                        let button_label = format!("Window##undock_win_{:?}", type_id);
                        if ui.small_button(&button_label) {
                            module.set_show_window(true);
                            module.set_show_menu(false);
                        }
                    }
                }
            }
        }

        // Render submenus as collapsing headers in window
        for (submenu_name, submenu_entries) in groups {
            if let Some(name) = submenu_name {
                let mut submenu_path = current_path.to_vec();
                submenu_path.push(name.clone());

                // if ui.button("Undock Group##undock_sub_win") {
                //     self.windowed_groups.insert(submenu_path.clone(), true);
                // }
                self.render_grouped_entries_in_window(
                    ui,
                    &submenu_entries,
                    depth + 1,
                    &submenu_path,
                );
            }
        }
    }

    pub fn render_menu(&mut self, ui: &dyn UiBackend) {
        if self.menu_cache.borrow().is_none() || *self.menu_dirty.borrow() {
            *self.menu_cache.borrow_mut() = Some(self.rebuild_menu_cache());
            *self.menu_dirty.borrow_mut() = false;
        }

        let cache = self.menu_cache.take().unwrap();
        let tracking_enabled = self.state_tracker.borrow().enabled;

        for (top_name, entries) in cache.top_level.iter() {
            let top_path = vec![top_name.clone()];
            let is_windowed = self
                .windowed_groups
                .borrow()
                .get(&top_path)
                .copied()
                .unwrap_or(false);

            if !is_windowed {
                if let Some(_menu) = ui.begin_menu(top_name) {
                    self.render_grouped_entries_tracked(
                        ui,
                        entries,
                        1,
                        &top_path,
                        tracking_enabled,
                    );
                    ui.separator();
                    if ui.button("Undock Group##undock_grp") {
                        self.windowed_groups
                            .borrow_mut()
                            .insert(top_path.clone(), true);
                    }
                }
            }
        }

        *self.menu_cache.borrow_mut() = Some(cache);
    }

    fn render_grouped_entries_tracked(
        &mut self,
        ui: &dyn UiBackend,
        entries: &[(Vec<String>, String)],
        depth: usize,
        current_path: &[String],
        tracking_enabled: bool,
    ) {
        // Similar to render_grouped_entries but with tracking
        let mut sorted_entries: Vec<_> = entries.to_vec();
        sorted_entries.sort_by(|a, b| {
            let weight_a = self
                .hacs
                .get(&a.1)
                .map(|m| m.borrow().menu_weight())
                .unwrap_or(0.0);
            let weight_b = self
                .hacs
                .get(&b.1)
                .map(|m| m.borrow().menu_weight())
                .unwrap_or(0.0);
            weight_b
                .partial_cmp(&weight_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (path, name_key) in sorted_entries {
            if depth >= path.len() {
                if let Some(module_rc) = self.hacs.get(&name_key) {
                    let mut module = module_rc.borrow_mut();
                    if module.is_menu_enabled() && !module.is_window_enabled() {
                        let type_id = module.nac_type_id();
                        if tracking_enabled {
                            if let Some(tracker) =
                                self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                            {
                                tracker.begin_render_menu();
                            }
                        }

                        module.render_menu(ui);

                        if tracking_enabled {
                            if let Some(tracker) =
                                self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                            {
                                tracker.stasis();
                            }
                        }

                        ui.separator();
                        let button_label = format!("Window##undock_{:?}", type_id);
                        if ui.small_button(&button_label) {
                            module.metadata_mut().undocked_from_menu = true;
                            module.set_show_window(true);
                            module.set_show_menu(false);
                        }
                    }
                }
            } else {
                // Handle submenus...
                let submenu_name = path[depth].clone();
                let mut submenu_path = current_path.to_vec();
                submenu_path.push(submenu_name.clone());

                let is_windowed = self
                    .windowed_groups
                    .borrow_mut()
                    .get(&submenu_path)
                    .copied()
                    .unwrap_or(false);

                if !is_windowed {
                    let submenu_entries: Vec<_> = entries
                        .iter()
                        .filter(|(p, _)| p.len() > depth && p[depth] == submenu_name)
                        .cloned()
                        .collect();

                    if !submenu_entries.is_empty() {
                        if let Some(_menu) = ui.begin_menu(&submenu_name) {
                            self.render_grouped_entries_tracked(
                                ui,
                                &submenu_entries,
                                depth + 1,
                                &submenu_path,
                                tracking_enabled,
                            );
                            ui.separator();
                        }
                    }
                }
            }
        }
    }

    pub fn render_window(&mut self, ui: &dyn UiBackend) {
        let keys: Vec<_> = self.hacs.keys().cloned().collect();
        // Simple sort by window weight
        let mut sorted = keys;
        sorted.sort_by(|a, b| {
            let wa = self
                .hacs
                .get(a)
                .map(|m| m.borrow().window_weight())
                .unwrap_or(0.0);
            let wb = self
                .hacs
                .get(b)
                .map(|m| m.borrow().window_weight())
                .unwrap_or(0.0);
            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let scale = ui.current_font_size() / 14.0;
        let tracking_enabled = self.state_tracker.borrow().enabled;

        for name_key in sorted {
            if let Some(module_rc) = self.hacs.get(&name_key) {
                let mut module = module_rc.borrow_mut();
                let mut show = module.is_window_enabled();
                let name = module.name().to_string();
                let type_id = module.nac_type_id();

                if show {
                    let metadata = module.metadata();
                    let saved_pos = if metadata.window_pos == [0.0, 0.0] {
                        [80.0 * scale, 0.0]
                    } else {
                        metadata.window_pos
                    };

                    let mut options = WindowOptions::new()
                        .with_position(saved_pos, WinCondition::FirstUseEver)
                        .with_resizable(true);

                    if metadata.auto_resize_window {
                        options = options.with_always_auto_resize(true);
                    } else {
                        let scaled_size = [
                            metadata.window_size[0] * scale,
                            metadata.window_size[1] * scale,
                        ];
                        options = options.with_size(scaled_size, WinCondition::FirstUseEver);
                    }

                    if let Some(_token) = ui.begin_window_simple(&name, &mut show, &options) {
                        if tracking_enabled {
                            if let Some(tracker) =
                                self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                            {
                                tracker.begin_render_window();
                            }
                        }

                        // If undocked from menu, render menu content in the window
                        if module.metadata().undocked_from_menu {
                            module.render_menu(ui);
                        } else {
                            module.render_window(ui);
                        }

                        if tracking_enabled {
                            if let Some(tracker) =
                                self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                            {
                                tracker.stasis();
                            }
                        }

                        let pos = ui.get_window_pos();
                        let size = ui.get_window_size();
                        let metadata_mut = module.metadata_mut();
                        metadata_mut.window_pos = pos.into();
                        metadata_mut.window_size = size.into();
                    }
                }

                if !show {
                    // Reset undocked flag when window is closed
                    module.metadata_mut().undocked_from_menu = false;
                    module.set_show_window(show);
                    module.set_show_menu(!show);
                }
            }
        }
    }

    pub fn render_draw(
        &mut self,
        ui: &dyn UiBackend,
        draw_list_fg: &mut dyn DrawList,
        draw_list_bg: &mut dyn DrawList,
    ) {
        self.triggered_hotkeys.borrow_mut().clear();
        *self.triggered_hotkeys.borrow_mut() = self.hotkey_manager.borrow_mut().poll_all(ui);

        let mut render_tree: HashMap<Vec<String>, Vec<String>> = HashMap::new();
        let mut independent: Vec<String> = Vec::new(); // Store Keys (Strings)
        let tracking_enabled = self.state_tracker.borrow().enabled;

        let keys: Vec<_> = self.hacs.keys().cloned().collect();

        let mut sorted = keys;
        sorted.sort_by(|a, b| {
            let wa = self
                .hacs
                .get(a)
                .map(|m| m.borrow().draw_weight())
                .unwrap_or(0.0);
            let wb = self
                .hacs
                .get(b)
                .map(|m| m.borrow().draw_weight())
                .unwrap_or(0.0);
            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
        });

        for name_key in &sorted {
            if let Some(module_rc) = self.hacs.get(name_key) {
                let module = module_rc.borrow();
                let type_id = module.nac_type_id();

                if !module.is_render_enabled() {
                    continue;
                }
                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                    {
                        tracker.qued();
                    }
                }

                let path = module.render_draw_path();
                if path.is_empty() {
                    independent.push(name_key.clone());
                } else {
                    let path: Vec<String> = path.iter().map(|s| s.to_string()).collect();
                    render_tree
                        .entry(path)
                        .or_insert_with(Vec::new)
                        .push(name_key.clone());
                }
            }
        }

        for name_key in independent {
            if let Some(module_rc) = self.hacs.get(&name_key) {
                let mut module = module_rc.borrow_mut();
                let type_id = module.nac_type_id();

                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                    {
                        tracker.begin_render_draw();
                    }
                }

                module.render_draw(ui, draw_list_fg, draw_list_bg);

                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                    {
                        tracker.stasis();
                    }
                }
            }
        }

        for (_path, name_keys) in render_tree {
            for name_key in name_keys {
                if let Some(module_rc) = self.hacs.get(&name_key) {
                    let mut module = module_rc.borrow_mut();
                    let type_id = module.nac_type_id();

                    if tracking_enabled {
                        if let Some(tracker) =
                            self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                        {
                            tracker.begin_render_draw();
                        }
                    }

                    module.render_draw(ui, draw_list_fg, draw_list_bg);

                    if tracking_enabled {
                        if let Some(tracker) =
                            self.state_tracker.borrow_mut().get_tracker_mut(&type_id)
                        {
                            tracker.stasis();
                        }
                    }
                }
            }
        }

        // Render state tracker window
        self.state_tracker.borrow_mut().render_window(ui);
    }

    pub fn render_windowed_groups(&mut self, ui: &dyn UiBackend) {
        if self.menu_cache.borrow().is_none() {
            return;
        }

        let windowed: Vec<_> = self
            .windowed_groups
            .borrow()
            .iter()
            .filter(|&(_, &is_open)| is_open)
            .map(|(path, _)| path.clone())
            .collect();

        let cache = self.menu_cache.take().unwrap();

        for path in windowed {
            // Find entries for this path
            let entries = self.find_entries_for_path(&cache, &path);
            if !entries.is_empty() {
                let window_title = path.join(" > ");
                self.render_group_window(ui, &window_title, &entries, &path);
            }
        }
        *self.menu_cache.borrow_mut() = Some(cache);
    }
}
