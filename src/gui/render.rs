use std::{any::{TypeId}, collections::{BTreeMap, HashMap}};
use imgui::{Condition, Ui, DrawListMut};
use crate::HaCKS;

impl HaCKS {

    // fn dock_btn_txt(&mut self, ui: &imgui::Ui) {
    //     let windowed =  format!("{}",{
    //         if self.metadata_window {
    //             "Dock"
    //         } else {
    //             "Undock"
    //         }
    //     }
    //     );
    //     if ui.button(windowed){
    //         self.metadata_window =!self.metadata_window;
    //     }
    // }

  
    fn render_group_window(&mut self, ui: &imgui::Ui, name: &str, entries: &[(Vec<String>, TypeId)], path: &[String]) {
        let mut show = true;
        let scale = ui.current_font_size()/14.0;
        ui.window(name)
            .opened(&mut show)
            .always_auto_resize(true)
            .position([80.0 * scale, 100.0], Condition::FirstUseEver)
            .build(|| {
                // Add dock button
                // if ui.button("Dock##dock_grp") {
                //     self.windowed_groups.insert(path.to_vec(), false);
                // }
                // ui.separator();
                self.render_grouped_entries_in_window(ui, entries, 1, path);
            });
        
        if !show {
            self.windowed_groups.borrow_mut().remove(path);
        }
    }
    
    fn render_grouped_entries_in_window(
        &mut self,
        ui: &Ui,
        entries: &[(Vec<String>, TypeId)],
        depth: usize,
        current_path: &[String],
    ) {
        let mut groups: BTreeMap<Option<String>, Vec<(Vec<String>, TypeId)>> = BTreeMap::new();
    
        for (path, type_id) in entries {
            if depth >= path.len() {
                groups.entry(None).or_default().push((path.clone(), *type_id));
            } else {
                let submenu_name = path[depth].clone();
                groups.entry(Some(submenu_name)).or_default().push((path.clone(), *type_id));
            }
        }
    
        // Render modules that end at this level
        if let Some(terminal_entries) = groups.remove(&None) {
            for (_path, type_id) in terminal_entries {
                if let Some(module_rc) = self.hacs.get(&type_id) {
                    // Mutable borrow for actions
                    let mut module = module_rc.borrow_mut();
                    if module.is_menu_enabled() && !module.is_window_enabled() {
                        let name = module.name().to_string();
                
                        ui.menu(name, || {
                            module.render_menu(ui); // render_menu can be called on mutable borrow if it mutates state
                        });
                
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
                self.render_grouped_entries_in_window(ui, &submenu_entries, depth + 1, &submenu_path);
            }
        }
    }
    
    pub fn render_menu(&mut self, ui: &Ui) {
        if self.menu_cache.borrow().is_none() || *self.menu_dirty.borrow() {
            *self.menu_cache.borrow_mut() = Some(self.rebuild_menu_cache());
            *self.menu_dirty.borrow_mut() = false;
        }

        let cache = self.menu_cache.take().unwrap();
        let tracking_enabled = self.state_tracker.borrow().enabled;
     
        for (top_name, entries) in cache.top_level.iter() {
            let top_path = vec![top_name.clone()];
            let is_windowed = self.windowed_groups.borrow().get(&top_path).copied().unwrap_or(false);
            
            if !is_windowed {
                ui.menu(top_name, || {
                    self.render_grouped_entries_tracked(ui, entries, 1, &top_path, tracking_enabled);
                    ui.separator();
                    if ui.button("Undock Group##undock_grp") {
                        self.windowed_groups.borrow_mut().insert(top_path.clone(), true);
                    }
                });
            }
        }
        
        *self.menu_cache.borrow_mut() = Some(cache);
    }

    fn render_grouped_entries_tracked(
        &mut self,
        ui: &Ui,
        entries: &[(Vec<String>, TypeId)],
        depth: usize,
        current_path: &[String],
        tracking_enabled: bool,
    ) {
        // Similar to render_grouped_entries but with tracking
        let mut sorted_entries: Vec<_> = entries.to_vec();
        sorted_entries.sort_by(|a, b| {
            let weight_a = self.hacs.get(&a.1).map(|m| m.borrow().menu_weight()).unwrap_or(0.0);
            let weight_b = self.hacs.get(&b.1).map(|m| m.borrow().menu_weight()).unwrap_or(0.0);
            weight_b.partial_cmp(&weight_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (path, type_id) in sorted_entries {
            if depth >= path.len() {
                if let Some(module_rc) = self.hacs.get(&type_id) {
                    let mut module = module_rc.borrow_mut();
                    if module.is_menu_enabled() && !module.is_window_enabled() {
                        if tracking_enabled {
                            if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                                tracker.begin_render_menu();
                            }
                        }
                        
                        module.render_menu(ui);
                        
                        if tracking_enabled {
                            if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                                tracker.end_render_menu();
                            }
                        }
                        
                        ui.separator();
                        let button_label = format!("Window##undock_{:?}", type_id);
                        if ui.small_button(&button_label) {
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

                let is_windowed = self.windowed_groups.borrow_mut().get(&submenu_path).copied().unwrap_or(false);

                if !is_windowed {
                    let submenu_entries: Vec<_> = entries.iter()
                        .filter(|(p, _)| p.len() > depth && p[depth] == submenu_name)
                        .cloned()
                        .collect();

                    if !submenu_entries.is_empty() {
                        ui.menu(&submenu_name, || {
                            self.render_grouped_entries_tracked(ui, &submenu_entries, depth + 1, &submenu_path, tracking_enabled);
                            ui.separator();
                        });
                    }
                }
            }
        }
    }

     pub fn render_window(&mut self, ui: &Ui) {
        let type_ids: Vec<_> = self.hacs.keys().copied().collect();
        let sorted: Vec<TypeId> = self.sort_by_weight(type_ids, |m| m.borrow().window_weight()).clone();
        let scale = ui.current_font_size() / 14.0;
        let tracking_enabled = self.state_tracker.borrow().enabled;

        for type_id in sorted {
            if let Some(module_rc) = self.hacs.get(&type_id) {
                let mut module = module_rc.borrow_mut();
                let mut show = module.is_window_enabled();
                let name = module.name().to_string();

                if show {
                    let metadata = module.metadata();
                    let saved_pos = if metadata.window_pos == [0.0, 0.0] {
                        [80.0 * scale, 0.0]
                    } else {
                        metadata.window_pos
                    };

                    let mut window = ui.window(name)
                        .opened(&mut show)
                        .resizable(true)
                        .position(saved_pos, Condition::FirstUseEver);

                    if metadata.auto_resize_window {
                        window = window.always_auto_resize(true);
                    } else {
                        let scaled_size = [
                            metadata.window_size[0] * scale,
                            metadata.window_size[1] * scale
                        ];
                        window = window.size(scaled_size, Condition::FirstUseEver);
                    }

                    if let Some(_token) = window.begin() {
                        if tracking_enabled {
                            if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                                tracker.begin_render_window();
                            }
                        }
                        
                        module.render_window(ui);
                        
                        if tracking_enabled {
                            if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                                tracker.end_render_window();
                            }
                        }

                        let pos = ui.window_pos();
                        let size = ui.window_size();
                        let metadata_mut = module.metadata_mut();
                        metadata_mut.window_pos = pos;
                        metadata_mut.window_size = size;
                    }
                }

                if !show {
                    module.set_show_window(show);
                    module.set_show_menu(!show);
                }
            }
        }
    }

    pub fn render_draw(
        &mut self,
        ui: &imgui::Ui,
        draw_list_fg: &mut DrawListMut,
        draw_list_bg: &mut DrawListMut,
    ) {
        self.triggered_hotkeys.borrow_mut().clear();
        *self.triggered_hotkeys.borrow_mut() = self.hotkey_manager.borrow_mut().poll_all(ui);

        let mut render_tree: HashMap<Vec<String>, Vec<TypeId>> = HashMap::new();
        let mut independent: Vec<TypeId> = Vec::new();
        let tracking_enabled = self.state_tracker.borrow().enabled;

        let type_ids: Vec<_> = self.hacs.keys().copied().collect();
        let sorted = self.sort_by_weight(type_ids, |m_rc| m_rc.borrow().draw_weight());

        for type_id in &sorted {
            if let Some(module_rc) = self.hacs.get(type_id) {
                let module = module_rc.borrow();
                if !module.is_render_enabled() {
                    continue;
                }

                let path = module.render_draw_path();
                if path.is_empty() {
                    independent.push(*type_id);
                } else {
                    let path: Vec<String> = path.iter().map(|s| s.to_string()).collect();
                    render_tree.entry(path).or_insert_with(Vec::new).push(*type_id);
                }
            }
        }

        for type_id in independent {
            if let Some(module_rc) = self.hacs.get(&type_id) {
                let mut module = module_rc.borrow_mut();
                
                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                        tracker.begin_render_draw();
                    }
                }
                
                module.render_draw(ui, draw_list_fg, draw_list_bg);
                
                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                        tracker.end_render_draw();
                    }
                }
            }
        }

        for (_path, type_ids) in render_tree {
            for type_id in type_ids {
                if let Some(module_rc) = self.hacs.get(&type_id) {
                    let mut module = module_rc.borrow_mut();
                    
                    if tracking_enabled {
                        if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                            tracker.begin_render_draw();
                        }
                    }
                    
                    module.render_draw(ui, draw_list_fg, draw_list_bg);
                    
                    if tracking_enabled {
                        if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                            tracker.end_render_draw();
                        }
                    }
                }
            }
        }

        // Render state tracker window
        self.state_tracker.borrow_mut().render_window(ui);
    }

    pub fn render_windowed_groups(&mut self, ui: &imgui::Ui) {

        if self.menu_cache.borrow().is_none() {
            return;
        }
    

        let windowed: Vec<_> = self.windowed_groups.borrow()
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