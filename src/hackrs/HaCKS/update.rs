use std::any::TypeId;
#[cfg(feature = "gui")]
use imgui::Ui;

pub enum TickTarget {
    All,
    Module(TypeId),
    WeightRange(f32, f32),
}

impl crate::HaCKS {

    #[cfg(feature = "gui")]
    pub fn before_render(&mut self, ui: &Ui) {
        let sorted = self.topological_sort_update();
        for type_id in sorted {
            if let Some(module) = self.hacs.get(&type_id) {
                module.borrow_mut().before_render(ui);
            }
        }
    }

    pub fn on_unload(&mut self) {
        let sorted = self.topological_sort_update();
        for type_id in sorted {
            if let Some(module) = self.hacs.get(&type_id) {
                module.borrow_mut().on_unload();
            }
        }
    }

    pub fn update(&mut self) {
        self.tick(TickTarget::All);
    }

    pub fn tick(&mut self, target: TickTarget) {
        let sorted = self.topological_sort_update();

        for type_id in sorted {
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
                    module.borrow_mut().update(self);
                }
            }
        }
    }

    pub fn tick_weight_range(&mut self, min: f32, max: f32) {
        self.tick(TickTarget::WeightRange(min, max));
    }

    pub fn tick_by_name(&mut self, name: &str) {
        if let Some((&type_id, _)) = self
            .hacs
            .iter()
            .find(|(_, m)| m.borrow().name() == name)
        {
            if let Some(module) = self.hacs.get(&type_id) {
                module.borrow_mut().update(self);
            }
        }
    }

    pub fn tick_all(&mut self) {
        for module in self.hacs.values() {
            module.borrow_mut().update(self);
        }
    }
}

