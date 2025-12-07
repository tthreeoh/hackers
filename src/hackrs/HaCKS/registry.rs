use std::{any::TypeId, cell::RefCell, rc::Rc};
// use anyhow::Result;
// use serde::{Deserialize, Serialize};
// use std::path::Path;
// use libloading::{Library, Symbol};

use crate::hack::HaCK;

// #[allow(unused,improper_ctypes_definitions)]
// type CreateHaCFn = unsafe extern "C" fn() -> *mut dyn HaCK;

use crate::hackrs::stable_abi::HackersModule_Ref;

pub struct DynamicHaC {
    pub root_module: HackersModule_Ref,
    pub type_id: TypeId,
    pub path: String,
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
        let type_id = TypeId::of::<T>();
        let name = module.name().to_string();

        self.hacs.insert(type_id, Rc::new(RefCell::new(module)));
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

        self.hacs.insert(type_id, module);

        // Register with state tracker
        self.state_tracker
            .borrow_mut()
            .register_module(type_id, name);
    }

    pub fn eject_module<T: HaCK + 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(module_rc) = self.hacs.remove(&type_id) {
            module_rc.borrow_mut().on_unload();
            self.menu_dirty = true.into();

            // Unregister from state tracker
            self.state_tracker.borrow_mut().unregister_module(&type_id);

            true
        } else {
            false
        }
    }

    pub fn eject_module_by_id(&mut self, type_id: TypeId) -> bool {
        if let Some(module_rc) = self.hacs.remove(&type_id) {
            module_rc.borrow_mut().on_unload();
            self.menu_dirty = true.into();
            true
        } else {
            false
        }
    }

    pub fn get<T: HaCK + 'static>(&self) -> Option<std::cell::Ref<'_, T>> {
        self.hacs
            .get(&TypeId::of::<T>())
            .map(|rc| std::cell::Ref::map(rc.borrow(), |m| m.as_any().downcast_ref::<T>().unwrap()))
    }

    pub fn get_mut<T: HaCK + 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        self.hacs.get(&TypeId::of::<T>()).map(|rc| {
            std::cell::RefMut::map(rc.borrow_mut(), |m| {
                m.as_any_mut().downcast_mut::<T>().unwrap()
            })
        })
    }

    pub fn get_mut_by_id(&self, id: TypeId) -> Option<std::cell::RefMut<'_, dyn HaCK>> {
        self.hacs.get(&id).map(|rc| rc.borrow_mut())
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
        use abi_stable::library::RootModule;

        // Get path as string for duplicate checking and storage
        let path_buf = path.as_ref().to_path_buf();
        let path_str = path_buf.to_string_lossy().to_string();

        // Check if already loaded
        for lib in &self.loaded_libs {
            if lib.path == path_str {
                println!("Plugin already loaded: {}", path_str);
                return Ok(());
            }
        }

        // Load the module
        // We use unsafe because loading code is inherently unsafe as it executes arbitrary code.
        let root_module = unsafe { HackersModule_Ref::load_from_file(path.as_ref()) }?;

        // Create the HaCK instance
        let stable_hack = root_module.create_hack()();

        // Wrap it in ForeignHaCK with default metadata
        let foreign_hack = ForeignHaCK {
            inner: stable_hack,
            metadata: crate::hackrs::HaCMetadata {
                name: std::borrow::Cow::Borrowed("ForeignModule"),
                description: std::borrow::Cow::Borrowed("External DLL Module"),
                category: std::borrow::Cow::Borrowed("External"),
                hotkeys: vec![],
                menu_weight: 0.0,
                window_weight: 0.0,
                draw_weight: 0.0,
                update_weight: 0.0,
                visible_in_gui: true,
                is_menu_enabled: true,
                is_window_enabled: true,
                is_render_enabled: false,
                is_update_enabled: true,
                auto_resize_window: true,
                window_pos: [0.0, 0.0],
                window_size: [0.0, 0.0],
                access_control: crate::access::AccessControl::new(
                    crate::access::AccessLevel::ReadWrite,
                ),
            },
        };
        let type_id = std::any::TypeId::of::<ForeignHaCK>(); // This is just a placeholder typeid since all ForeignHaCKs share the same wrapper type.
                                                             // We might need a unique ID for each loaded module if we want them distinct in HashMap?
                                                             // But HashMap key is TypeId.
                                                             // If we load multiple DLLs, they will all be ForeignHaCK.
                                                             // We should wrap ForeignHaCK in another struct or use a mapping?
                                                             // Actually, the registry uses `TypeId` of the `HaCK` generic K.
                                                             // If all dynamic modules are `ForeignHaCK`, we can only have one active?
                                                             // That's a limitation of current architecture.
                                                             // We can create a unique wrapper type per load? Not possible at runtime.
                                                             // We can change `hacs` map to key by `String` (Name) instead of `TypeId`?
                                                             // `hacs: HashMap<TypeId, Rc<RefCell<dyn HaCK>>>`.

        // For now, let's assume one dynamic module or that we only need one "ForeignHaCK" type.
        // To support multiple, we would need to redesign HaCKS to not rely solely on static TypeId for storage,
        // or use `Box<dyn HaCK>` and store by Name.

        // However, `register_boxed` uses `nac_type_id()` which returns `ForeignHaCK`'s type ID.
        // So yes, multiple DLLs will conflict if they use the same ForeignHaCK wrapper.
        // But we can still load it and see if it works for one.

        let name = foreign_hack.inner.name().to_string();

        // Store path and library
        self.loaded_libs.push(DynamicHaC {
            root_module,
            type_id,
            path: path_str,
        });

        // Store in dynamic_modules vector for multi-DLL support
        self.dynamic_modules
            .push(Rc::new(RefCell::new(foreign_hack)));

        Ok(())
    }

    /// Unload a dynamic module by index
    pub fn unload_dynamic(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.dynamic_modules.len() {
            return Err("Index out of bounds".into());
        }

        // Call on_unload before removing
        self.dynamic_modules[index].borrow_mut().on_unload();

        // Remove from both vectors
        self.dynamic_modules.remove(index);
        if index < self.loaded_libs.len() {
            self.loaded_libs.remove(index);
        }

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

    /// Reload a dynamic module by index
    pub fn reload_dynamic(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.loaded_libs.len() {
            return Err("Index out of bounds".into());
        }

        // Get the library path (we need to store this)
        // For now, this is a limitation - we can't reload without knowing the path
        Err("Reload not yet implemented - path tracking needed".into())
    }

    // pub fn reload_module<P: AsRef<Path>>(&mut self, path: P, type_id: TypeId) -> Result<()> {
    //     // unsafe {
    //     //     self.unload_dynamic(type_id)?;
    //     //     self.load_dynamic(path)
    //     // }
    // }
}
