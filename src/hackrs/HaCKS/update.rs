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
    pub fn before_render(&mut self,ui: &Ui) {
        let sorted = self.topological_sort_update();
        for type_id in sorted {
            if let Some(module) = self.hacs.get_mut(&type_id) {
                module.before_render(ui);
            }
        }
     
    }

    pub fn on_unload(&mut self) {
        let sorted = self.topological_sort_update();
        for type_id in sorted {
            if let Some(module) = self.hacs.get_mut(&type_id) {
                module.on_unload();
            }
        }
     
    }


    
    pub fn update(&mut self) {
        self.tick(TickTarget::All);
    }

    pub fn tick(&mut self, target: TickTarget) {
        let sorted = self.topological_sort_update();
        let self_ptr = self as *const crate::HaCKS; // raw pointer for read-only escape

        for type_id in sorted {
            let should_update = {
                let module = &self.hacs[&type_id];
                let weight = module.update_weight();
                match &target {
                    TickTarget::All => true,
                    TickTarget::Module(id) if module.nac_type_id() == *id => true,
                    TickTarget::WeightRange(min, max) => (*min..=*max).contains(&weight),
                    _ => false,
                }
            };

            if should_update {
                if let Some(module) = self.hacs.get_mut(&type_id) {
                    let hacs_ref = unsafe { &*self_ptr }; 
                    module.update(hacs_ref);
                }
            }
        }
    }


    // pub fn tick_module<T: 'static + HaC>(&mut self) {
    //     self.tick(TickTarget::Module(TypeId::of::<T>()));
    // }

    pub fn tick_weight_range(&mut self, min: f32, max: f32) {
        self.tick(TickTarget::WeightRange(min, max));
    }

    pub fn tick_by_name(&mut self, name: &str) {
        let type_id_opt = self
            .hacs
            .iter()
            .find_map(|(id, m)| (m.name() == name).then_some(*id));

        if let Some(type_id) = type_id_opt {
            let self_ptr = self as *const crate::HaCKS;
            if let Some(module) = self.hacs.get_mut(&type_id) {
                let hacs_ref = unsafe { &*self_ptr };
                module.update(hacs_ref);
            }
        }
    }

    pub fn tick_all(&mut self) {
        let ptr = self as *const crate::HaCKS;
        for module in self.hacs.values_mut() {
            let hacs_ref = unsafe { &*ptr };
            module.update(hacs_ref);
        }
    }
}
