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

    pub fn init_all(&self) {
        let sorted = self.topological_sort_update();
        let tracking_enabled = self.state_tracker.borrow().enabled;
        
        for type_id in sorted {
            if let Some(module_rc) = self.hacs.get(&type_id) {
                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                        tracker.begin_init();
                    }
                }
                
                module_rc.borrow_mut().init();
                
                if tracking_enabled {
                    if let Some(tracker) = self.state_tracker.borrow_mut().get_tracker_mut(&type_id) {
                        tracker.end_init();
                    }
                }
            }
        }
    }
    
    pub fn exit_all(&self) {
        for module_rc in self.hacs.values() {
            module_rc.borrow_mut().exit();
        }
    }

}