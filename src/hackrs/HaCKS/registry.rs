use std::{any::TypeId, cell::RefCell, rc::Rc};
// use anyhow::Result;
// use serde::{Deserialize, Serialize};
// use std::path::Path;
// use libloading::{Library, Symbol};

use crate::hack::HaCK;

// #[allow(unused,improper_ctypes_definitions)]
// type CreateHaCFn = unsafe extern "C" fn() -> *mut dyn HaCK;

use crate::hackrs::stable_abi::HackersModule_Ref;

use libloading::Library;

pub struct DynamicHaC {
    pub root_module: HackersModule_Ref,
    pub type_id: TypeId,
    pub path: String,
    pub library: Rc<Library>, // Keep library alive
}

impl std::fmt::Debug for DynamicHaC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynamicHaC")
    }
}
// pub trait RcRefExtract<T> {
//     fn take_rc_ref(&mut self) -> Rc<RefCell<T>>;
// }

// impl<T> RcRefExtract<T> for Rc<RefCell<T>> {
//     fn take_rc_ref(&mut self) -> Rc<RefCell<T>> {
//         Rc::clone(self)
//     }
// }

// #[macro_export]
// macro_rules! impl_dummy_nac_boilerplate {
//     () => {
//         fn is_menu_enabled(&self) -> bool { false }
//         fn is_window_enabled(&self) -> bool { false }
//         fn set_show_menu(&mut self, _: bool) -> bool { false }
//         fn set_show_window(&mut self, _: bool) -> bool { false }

//         fn metadata(&self) -> &$crate::HaCMetadata {
//             static META: $crate::HaCMetadata = $crate::HaCMetadata {
//                 name: std::borrow::Cow::Borrowed("Dummy"),
//                 description: std::borrow::Cow::Borrowed("Placeholder module"),
//                 category: std::borrow::Cow::Borrowed("System"),
//                 hotkeys: Vec::new(),
//                 menu_weight: 0.0,
//                 window_weight: 0.0,
//                 draw_weight: 0.0,
//                 update_weight: 0.0,
//                 visible_in_gui: false,
//                 is_menu_enabled: false,
//                 is_window_enabled: false,
//                 is_render_enabled: false,
//                 is_update_enabled: false,
//                 auto_resize_window: true,
//                 window_pos: crate::metadata::default_window_pos(),
//                 window_size: crate::metadata::default_window_size(),
//             };
//             &META
//         }

//         fn metadata_mut(&mut self) -> &mut $crate::metadata::HaCMetadata {
//             panic!("DummyHaC metadata_mut called!");
//         }
//         fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
//             serde_json::to_value(self)
//         }
//         fn is_render_enabled(&self) -> bool { false }
//         fn is_update_enabled(&self) -> bool { false }
//         fn menu_weight(&self) -> f32 { 0.0 }
//         fn window_weight(&self) -> f32 { 0.0 }
//         fn draw_weight(&self) -> f32 { 0.0 }
//         fn update_weight(&self) -> f32 { 0.0 }
//         fn set_menu_weight(&mut self, _: f32) {}
//         fn set_window_weight(&mut self, _: f32) {}
//         fn set_draw_weight(&mut self, _: f32) {}
//         fn set_update_weight(&mut self, _: f32) {}
//         fn set_render_enabled(&mut self, _: bool) {}
//         fn set_update_enabled(&mut self, _: bool) {}

//         fn as_any(&self) -> &dyn std::any::Any { self }
//         fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
//     };
// }
// #[derive(Clone, Serialize, Deserialize, PartialEq)]
// struct DummyHaC;
// impl HaCK for DummyHaC {
//     fn name(&self)->&str{
//         "DummyHaC"
//     }
//     fn nac_type_id(&self) -> TypeId { TypeId::of::<DummyHaC>() }
//     fn update(&mut self,_hacs: &crate::HaCKS) {}
//     impl_dummy_nac_boilerplate!();
// }
impl crate::HaCKS {
    pub fn register<T: HaCK + 'static>(&mut self, module: T) {
        let name = module.name().to_string();
        let type_id = TypeId::of::<T>();

        self.hacs
            .insert(name.clone(), Rc::new(RefCell::new(module)));
        self.menu_dirty = true.into();

        // Register with state tracker
        self.state_tracker
            .borrow_mut()
            .register_module(type_id, name);
    }

    pub fn register_boxed(&mut self, module: Rc<RefCell<dyn HaCK>>) {
        let (type_id, name) = {
            let m_ref = module.borrow();
            (m_ref.nac_type_id(), m_ref.name().to_string())
        };

        self.hacs.insert(name.clone(), module);

        // Register with state tracker
        self.state_tracker
            .borrow_mut()
            .register_module(type_id, name);
    }

    pub fn eject_module<T: HaCK + 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        // Find key by TypeId
        let key = self.hacs.iter().find_map(|(k, v)| {
            if v.borrow().nac_type_id() == type_id {
                Some(k.clone())
            } else {
                None
            }
        });

        if let Some(k) = key {
            if let Some(module_rc) = self.hacs.remove(&k) {
                module_rc.borrow_mut().on_unload();
                self.menu_dirty = true.into();

                // Unregister from state tracker
                self.state_tracker.borrow_mut().unregister_module(&type_id);

                return true;
            }
        }
        false
    }

    pub fn eject_module_by_id(&mut self, type_id: TypeId) -> bool {
        // Find key by TypeId
        let key = self.hacs.iter().find_map(|(k, v)| {
            if v.borrow().nac_type_id() == type_id {
                Some(k.clone())
            } else {
                None
            }
        });

        if let Some(k) = key {
            if let Some(module_rc) = self.hacs.remove(&k) {
                module_rc.borrow_mut().on_unload();
                self.menu_dirty = true.into();
                return true;
            }
        }
        false
    }

    pub fn get<T: HaCK + 'static>(&self) -> Option<std::cell::Ref<'_, T>> {
        // Warning: O(N) lookup by type
        // This is safe but slower than O(1)
        self.hacs.values().find_map(|rc| {
            let borrow = rc.borrow();
            if borrow.nac_type_id() == TypeId::of::<T>() {
                Some(std::cell::Ref::map(borrow, |m| {
                    m.as_any().downcast_ref::<T>().unwrap()
                }))
            } else {
                None
            }
        })
    }

    pub fn get_mut<T: HaCK + 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        // Warning: O(N) lookup by type
        self.hacs.values().find_map(|rc| {
            // Check type without keeping borrow
            let is_match = rc.borrow().nac_type_id() == TypeId::of::<T>();
            if is_match {
                Some(std::cell::RefMut::map(rc.borrow_mut(), |m| {
                    m.as_any_mut().downcast_mut::<T>().unwrap()
                }))
            } else {
                None
            }
        })
    }

    pub fn get_mut_by_id(&self, id: TypeId) -> Option<std::cell::RefMut<'_, dyn HaCK>> {
        self.hacs.values().find_map(|rc| {
            if rc.borrow().nac_type_id() == id {
                Some(rc.borrow_mut())
            } else {
                None
            }
        })
    }

    pub fn hacs(&self) -> impl Iterator<Item = std::cell::Ref<'_, dyn HaCK>> + '_ {
        self.hacs.values().map(|rc| rc.borrow())
    }

    pub fn hacs_mut(&self) -> impl Iterator<Item = std::cell::RefMut<'_, dyn HaCK>> + '_ {
        self.hacs.values().map(|rc| rc.borrow_mut())
    }

    /// Dynamically load a `.dll` / `.so` / `.dylib` module and register it.
    pub fn load_dynamic<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::hackrs::stable_abi::ForeignHaCK;
        use crate::hackrs::stable_abi::HackersModule_Ref;
        // use abi_stable::library::RootModule; // Not needed

        // Get path as string for duplicate checking and storage
        let path_buf = path.as_ref().to_path_buf();
        let path_str = path_buf.to_string_lossy().to_string();

        // Check if already loaded by path in loaded_libs hashmap
        for lib in self.loaded_libs.borrow().values() {
            if lib.path == path_str {
                println!("Plugin already loaded from path: {}", path_str);
                return Ok(());
            }
        }

        // Load multiple copies of the same library logic
        // We use libloading directly to control the lifetime and bypass name caching
        let library = Rc::new(unsafe { libloading::Library::new(path.as_ref())? });

        // Load the LibHeader source explicitly
        let header: &abi_stable::library::LibHeader = unsafe {
            let sym_name = abi_stable::library::ROOT_MODULE_LOADER_NAME;
            // libloading expects bytes (without null terminator if os specific? No, standard is bytes)
            // But ROOT_MODULE_LOADER_NAME is a string.
            let sym: libloading::Symbol<&abi_stable::library::LibHeader> =
                library.get(sym_name.as_bytes())?;
            *sym
        };

        // Initialize the root module
        let root_module = unsafe { header.init_root_module::<HackersModule_Ref>() }?;

        // Create the HaCK instance
        let stable_hack = root_module.create_hack()();

        // Wrap in ForeignHaCK
        let foreign_hack = ForeignHaCK::new(stable_hack);

        let name = foreign_hack.inner.name().to_string();
        let type_id = foreign_hack.nac_type_id();

        // Check if module with same name already exists in main registry
        if self.hacs.contains_key(&name) {
            return Err(format!(
                "Module with name '{}' already registered. Cannot load duplicate from '{}'",
                name, path_str
            )
            .into());
        }

        // Insert into hacs registry
        self.hacs
            .insert(name.clone(), Rc::new(RefCell::new(foreign_hack)));
        self.menu_dirty = true.into();

        // Track in loaded_libs map
        self.loaded_libs.borrow_mut().insert(
            name.clone(),
            DynamicHaC {
                root_module,
                type_id,
                path: path_str.clone(),
                library,
            },
        );

        println!("Loaded dynamic module '{}' from: {}", name, path_str);

        Ok(())
    }

    /// Unload a dynamic module by name
    pub fn unload_dynamic(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut loaded_libs = self.loaded_libs.borrow_mut();

        if !loaded_libs.contains_key(name) {
            return Err(format!("Plugin '{}' not found in loaded libraries", name).into());
        }

        // Remove from hacs
        if let Some(module_rc) = self.hacs.remove(name) {
            module_rc.borrow_mut().on_unload();
            self.menu_dirty = true.into();
            println!("Unloaded dynamic module '{}'", name);
        } else {
            println!(
                "Warning: Module '{}' found in loaded_libs but not in main registry",
                name
            );
        }

        // Remove from loaded_libs
        loaded_libs.remove(name);

        Ok(())
    }

    /// Load all DLLs from a directory
    pub fn load_plugins_from_folder<P: AsRef<std::path::Path>>(
        &mut self,
        folder: P,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut loaded_count = 0;

        let entries = std::fs::read_dir(folder)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only load .dll files on Windows
            if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                match self.load_dynamic(&path) {
                    Ok(_) => {
                        println!("Loaded plugin: {:?}", path);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(loaded_count)
    }

    /// Reload a dynamic module by name
    pub fn reload_dynamic(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // if index >= self.loaded_libs.len() {
        //     return Err("Index out of bounds".into());
        // }

        // Get the library path (we need to store this)
        // For now, this is a limitation - we can't reload without knowing the path
        Err(format!(
            "Reload not yet implemented for '{}' - path tracking needed",
            name
        )
        .into())
    }

    // pub fn reload_module<P: AsRef<Path>>(&mut self, path: P, type_id: TypeId) -> Result<()> {
    //     // unsafe {
    //     //     self.unload_dynamic(type_id)?;
    //     //     self.load_dynamic(path)
    //     // }
    // }
}
