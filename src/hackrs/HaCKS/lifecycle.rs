use std::any::{TypeId};

use crate::{HaCKS};

impl HaCKS {
    pub fn get_init_data<T: 'static>(&self) -> Option<&T> {
        self.init_data
            .get(&TypeId::of::<T>())
            .and_then(|d| d.downcast_ref::<T>())
    }

    pub fn set_init_data<T: Send + 'static>(&mut self, data: T) {
        self.init_data.insert(TypeId::of::<T>(), Box::new(data));
    }

    pub fn init_all(&mut self) {
        let sorted = self.topological_sort_update();
        for type_id in sorted {
            if let Some(module) = self.hacs.get_mut(&type_id) {
                module.init();
            }
        }
    }
    pub fn exit_all(&mut self) {
        for module in self.hacs.values_mut() {
            module.exit();
        }
    }

}