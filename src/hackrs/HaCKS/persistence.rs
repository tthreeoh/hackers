use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{HaCK, HaCKS};

impl HaCKS {

    pub fn with_modules(modules: Vec<Rc<RefCell<dyn HaCK>>>) -> Self {
        let mut container = HaCKS::new();
        for module in modules {
            container.register_boxed(module);
        }
        container
    }

    pub fn load_from_file<F, P: AsRef<std::path::Path>>(
        path: P,
        mut load_settings_fn: F
    ) -> (Self, HashMap<String, serde_json::Value>, Vec<String>)
    where
        F: FnMut(&HashMap<String, serde_json::Value>) -> Vec<Rc<RefCell<dyn HaCK>>>,
    {
        let mut container = HaCKS::new();
        let mut extra_settings = HashMap::new();
    
        let debug_path = path.as_ref().parent()
            .map(|p| p.join("persistence_debug.txt"))
            .unwrap_or_else(|| std::path::PathBuf::from("persistence_debug.txt"));
    
        let mut debug_log = format!("load_from_file: Reading from {:?}\n", path.as_ref());
    
        if let Ok(contents) = std::fs::read_to_string(&path) {
            debug_log.push_str(&format!("File read successfully, {} bytes\n", contents.len()));
    
            if let Ok(settings) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&contents) {
                debug_log.push_str(&format!("Parsed {} settings\n", settings.len()));
                extra_settings = settings.clone();
    
                for module in load_settings_fn(&settings) {
                    container.register_boxed(module);
                }
    
                debug_log.push_str(&format!("Registered {} modules\n", container.hacs.len()));
                let _ = std::fs::write(&debug_path, &debug_log);
    
                return (container, extra_settings, vec![]);
            } else {
                debug_log.push_str("Failed to parse JSON\n");
            }
        } else {
            debug_log.push_str("Failed to read file\n");
        }
    
        debug_log.push_str("Using defaults\n");
        for module in load_settings_fn(&HashMap::new()) {
            container.register_boxed(module);
        }
        debug_log.push_str(&format!("Created {} default modules\n", container.hacs.len()));
        let _ = std::fs::write(&debug_path, debug_log);
    
        (container, extra_settings, vec![])
    }
    

    pub fn save_to_file<F, P: AsRef<std::path::Path>>(
        &self,
        path: P,
        save_settings_fn: F,
        extra_settings: Option<&HashMap<String, serde_json::Value>>
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&HashMap<TypeId, Rc<RefCell<dyn HaCK>>>) -> HashMap<String, serde_json::Value>,
    {
        let mut settings = save_settings_fn(&self.hacs);
        if let Some(extra) = extra_settings {
            for (key, value) in extra {
                settings.insert(key.clone(), value.clone());
            }
        }

        let pretty_config = ron::ser::PrettyConfig::new().depth_limit(4).struct_names(true);
        let contents = ron::ser::to_string_pretty(&settings, pretty_config)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn reset_all_modules<F>(&mut self, create_modules_fn: F)
        where
            F: Fn() -> Vec<Rc<RefCell<dyn HaCK>>>,
        {
            let init_data = std::mem::take(&mut self.init_data);
            self.hacs.clear();

            for module in create_modules_fn() {
                self.register_boxed(module);
            }

            self.init_data = init_data;
            self.menu_dirty = true;
        }

        pub fn reset_module<T: HaCK + 'static, F>(&mut self, create_modules_fn: F) -> bool
        where
            F: Fn() -> Vec<Rc<RefCell<dyn HaCK>>>,
        {
            let type_id = TypeId::of::<T>();
            if self.hacs.remove(&type_id).is_some() {
                for module in create_modules_fn() {
                    if module.borrow().nac_type_id() == type_id {
                        self.register_boxed(module);
                        self.menu_dirty = true;
                        return true;
                    }
                }
            }
            false
        }

        pub fn reset_module_by_id<F>(&mut self, type_id: TypeId, create_modules_fn: F) -> bool
        where
            F: Fn() -> Vec<Rc<RefCell<dyn HaCK>>>,
        {
            if self.hacs.remove(&type_id).is_some() {
                for module in create_modules_fn() {
                    if module.borrow().nac_type_id() == type_id {
                        self.register_boxed(module);
                        self.menu_dirty = true;
                        return true;
                    }
                }
            }
            false
        }

}
