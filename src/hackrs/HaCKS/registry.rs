use std::any::TypeId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use libloading::{Library, Symbol};


use crate::{
    hack::HaCK};

#[allow(unused,improper_ctypes_definitions)]
type CreateHaCFn = unsafe extern "C" fn() -> *mut dyn HaCK;


pub struct DynamicHaC {
    pub lib: Library,
    pub instance: Box<dyn HaCK>,
    pub type_id: TypeId,
}

impl std::fmt::Debug for DynamicHaC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynamicHaC")
    }
}

pub trait BoxExtract {
    fn take_box(&mut self) -> Box<dyn HaCK>;
}

impl BoxExtract for Box<dyn HaCK> {
    fn take_box(&mut self) -> Box<dyn HaCK> {
        // NOTE: This is a hacky but valid move pattern using swap
        std::mem::replace(self, Box::new(DummyHaC))
    }
}





#[macro_export]
macro_rules! impl_dummy_nac_boilerplate {
    () => {
        fn is_menu_enabled(&self) -> bool { false }
        fn is_window_enabled(&self) -> bool { false }
        fn set_show_menu(&mut self, _: bool) -> bool { false }
        fn set_show_window(&mut self, _: bool) -> bool { false }

        fn metadata(&self) -> &$crate::HaCMetadata {
            static META: $crate::HaCMetadata = $crate::HaCMetadata {
                name: std::borrow::Cow::Borrowed("Dummy"),
                description: std::borrow::Cow::Borrowed("Placeholder module"),
                category: std::borrow::Cow::Borrowed("System"),
                hotkeys: Vec::new(),
                menu_weight: 0.0,
                window_weight: 0.0,
                draw_weight: 0.0,
                update_weight: 0.0,
                visible_in_gui: false,
                is_menu_enabled: false,
                is_window_enabled: false,
                is_render_enabled: false,
                is_update_enabled: false,
                auto_resize_window: true,
                window_pos: crate::metadata::default_window_pos(),
                window_size: crate::metadata::default_window_size(),
            };
            &META
        }

        fn metadata_mut(&mut self) -> &mut $crate::metadata::HaCMetadata {
            panic!("DummyHaC metadata_mut called!");
        }
        fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
            serde_json::to_value(self)
        }
        fn is_render_enabled(&self) -> bool { false }
        fn is_update_enabled(&self) -> bool { false }
        fn menu_weight(&self) -> f32 { 0.0 }
        fn window_weight(&self) -> f32 { 0.0 }
        fn draw_weight(&self) -> f32 { 0.0 }
        fn update_weight(&self) -> f32 { 0.0 }
        fn set_menu_weight(&mut self, _: f32) {}
        fn set_window_weight(&mut self, _: f32) {}
        fn set_draw_weight(&mut self, _: f32) {}
        fn set_update_weight(&mut self, _: f32) {}
        fn set_render_enabled(&mut self, _: bool) {}
        fn set_update_enabled(&mut self, _: bool) {}

        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    };
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct DummyHaC;
impl HaCK for DummyHaC {
    fn name(&self)->&str{
        "DummyHaC"
    }
    fn nac_type_id(&self) -> TypeId { TypeId::of::<DummyHaC>() }
    fn update(&mut self,_hacs: &crate::HaCKS) {}
    impl_dummy_nac_boilerplate!();
}

impl crate::HaCKS {
    pub fn register<T: HaCK + 'static>(&mut self, module: T) {
        self.hacs.insert(TypeId::of::<T>(), Box::new(module));
        self.menu_dirty =  true;
    }

    pub fn register_boxed(&mut self, module: Box<dyn HaCK>) {
        self.hacs.insert(module.nac_type_id(), module);
    }

    pub fn eject_module<T: HaCK + 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(mut module) = self.hacs.remove(&type_id) {
            module.on_unload();
            self.menu_dirty = true;
            true
        } else {
            false
        }
    }
    
    pub fn eject_module_by_id(&mut self, type_id: TypeId) -> bool {
        if let Some(mut module) = self.hacs.remove(&type_id) {
            module.on_unload();
            self.menu_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn get<T: HaCK + 'static>(&self) -> Option<&T> {
        self.hacs
            .get(&TypeId::of::<T>())
            .and_then(|m| m.as_any().downcast_ref::<T>())
    }

    pub fn get_mut<T: HaCK + 'static>(&mut self) -> Option<&mut T> {
        self.hacs
            .get_mut(&TypeId::of::<T>())
            .and_then(|m| m.as_any_mut().downcast_mut::<T>())
    }

    pub fn get_mut_by_id(&mut self, id: TypeId) -> Option<&mut Box<dyn HaCK>> {
        self.hacs.get_mut(&id)
    }

    pub fn hacs(&self) -> impl Iterator<Item = &dyn HaCK> {
        self.hacs.values().map(|boxed| boxed.as_ref())
    }

    pub fn hacs_mut(&mut self) -> impl Iterator<Item = &mut dyn HaCK> {
        self.hacs.values_mut().map(|boxed| boxed.as_mut())
    }

     /// Dynamically load a `.dll` / `.so` / `.dylib` module and register it.
     pub unsafe fn load_dynamic<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // Load the dynamic library
        let lib = Library::new(path.as_ref())?;

        // Define the expected exported constructor type
        #[allow(improper_ctypes_definitions)]
        type CreateHaCFn = unsafe extern "C" fn() -> *mut dyn HaCK;

        // Load the constructor symbol
        let constructor: Symbol<CreateHaCFn> = lib.get(b"create_nac")?;
        let raw = constructor();

        // Take ownership of the instance
        let boxed: Box<dyn HaCK> = Box::from_raw(raw);
        let type_id = boxed.nac_type_id();

        // Register it in the runtime
        self.register_boxed(boxed);

        // Retrieve a cloned handle for tracking
        let instance = self
            .get_mut_by_id(type_id)
            .map(|m| m.take_box())
            .unwrap_or_else(|| panic!("Module was not registered correctly"));

        // Push the library + boxed module reference
        self.loaded_libs.push(DynamicHaC {
            lib,
            instance,
            type_id,
        });
        Ok(())
    }
    
    /// Unload a dynamically loaded HaC and remove it from the container.
    pub unsafe fn unload_dynamic(&mut self, type_id: TypeId) -> Result<()> {
        // First, eject the module itself (this calls on_unload)
        if !self.eject_module_by_id(type_id) {
            anyhow::bail!("Module not found in container");
        }

        // Remove from loaded_libs to drop the library handle (which unloads the DLL)
        if let Some(pos) = self.loaded_libs.iter().position(|m| m.type_id == type_id) {
            let dynamic = self.loaded_libs.remove(pos);
            drop(dynamic); // drops library safely
            Ok(())
        } else {
            anyhow::bail!("No dynamic library entry found for module");
        }
    }

    pub fn reload_module<P: AsRef<Path>>(&mut self, path: P, type_id: TypeId) -> Result<()> {
        unsafe {
            self.unload_dynamic(type_id)?;
            self.load_dynamic(path)
        }
    }

}


