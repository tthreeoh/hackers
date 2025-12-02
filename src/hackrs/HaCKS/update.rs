use std::any::TypeId;
#[cfg(any(feature = "gui",feature = "ui-imgui"))]
use imgui::Ui;

pub enum TickTarget {
    All,
    Module(TypeId),
    WeightRange(f32, f32),
}

impl crate::HaCKS {

    #[cfg(any(feature = "gui",feature = "ui-imgui"))]
    pub fn before_render(&self, ui: &Ui) {
        let sorted = self.topological_sort_update();
        let tracking_enabled = self.state_tracker.borrow().enabled;
        if tracking_enabled {
            for type_id in &sorted {
                if let Some(_module) = self.hacs.get(&type_id) {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                        tracker.qued();
                    }
                }
            }
        }
        for type_id in &sorted {
            if let Some(module) = self.hacs.get(&type_id) {
                module.borrow_mut().before_render(ui);
            }
        }
        if tracking_enabled {
        for type_id in &sorted {
            if let Some(_module) = self.hacs.get(&type_id) {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                        tracker.stasis();
                    }
                }
            }
        }
    }

    pub fn on_unload(&self) {
        let sorted = self.topological_sort_update();
        for type_id in sorted {
            if let Some(module) = self.hacs.get(&type_id) {
                module.borrow_mut().on_unload();
            }
        }
    }

    pub fn update(&self) {
        self.sync_modules();
        self.tick(TickTarget::All);
    }

    pub fn tick(&self, target: TickTarget) {
        let sorted = self.topological_sort_update();
        let tracking_enabled = self.state_tracker.borrow().enabled;
        for type_id in &sorted {
            let should_update = {
                let module = self.hacs.get(&type_id).unwrap();
                let weight = module.borrow().update_weight();
                match &target {
                    TickTarget::All => true,
                    TickTarget::Module(id) if module.borrow().nac_type_id() == *id => true,
                    TickTarget::WeightRange(min, max) => (*min..=*max).contains(&weight),
                    _ => false,
                }
            };
            if should_update {
                if tracking_enabled {
                    if let Some(_module) = self.hacs.get(&type_id) {
                        if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                            tracker.qued();
                        }
                    }
                }
            }
        }

        for type_id in &sorted {
            let should_update = {
                let module = self.hacs.get(&type_id).unwrap();
                let weight = module.borrow().update_weight();
                match &target {
                    TickTarget::All => true,
                    TickTarget::Module(id) if module.borrow().nac_type_id() == *id => true,
                    TickTarget::WeightRange(min, max) => (*min..=*max).contains(&weight),
                    _ => false,
                }
            };

            if should_update {
                if let Some(module) = self.hacs.get(&type_id) {
                    // Track update lifecycle
                    if tracking_enabled {
                        if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                            tracker.begin_update();
                        }
                    }
                    
                    module.borrow_mut().update(self);
                    
                    if tracking_enabled {
                        if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                            tracker.end_update();
                        }
                    }
                }
            }
        }
        
        if tracking_enabled {
            for type_id in &sorted {
                let should_update = {
                    let module = self.hacs.get(&type_id).unwrap();
                    let weight = module.borrow().update_weight();
                    match &target {
                        TickTarget::All => true,
                        TickTarget::Module(id) if module.borrow().nac_type_id() == *id => true,
                        TickTarget::WeightRange(min, max) => (*min..=*max).contains(&weight),
                        _ => false,
                    }
                };
                if should_update {
                    if let Some(_module) = self.hacs.get(&type_id) {
                        if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                            tracker.stasis();
                        }
                    }
                }
            }
        }

        // Auto-flatten if needed
        if tracking_enabled {
            self.state_tracker.borrow_mut().flatten_if_needed();
        }
    }

    pub fn tick_weight_range(&self, min: f32, max: f32) {
        self.tick(TickTarget::WeightRange(min, max));
    }

    // pub fn tick_by_name(&self, name: &str) {
    //     if let Some((&type_id, _)) = self
    //         .hacs
    //         .iter()
    //         .find(|(_, m)| m.borrow().name() == name)
    //     {
    //         if let Some(module) = self.hacs.get(&type_id) {
    //             module.borrow_mut().update(self);
    //         }
    //     }
    // }

    // pub fn tick_all(&self) {
    //     for module in self.hacs.values() {
    //         module.borrow_mut().update(self);
    //     }
    // }
}