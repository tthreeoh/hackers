use std::{any::TypeId, cell::RefCell, rc::Rc};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use libloading::{Library, Symbol};


use crate::{
    hack::HaCK};

// #[allow(unused,improper_ctypes_definitions)]
// type CreateHaCFn = unsafe extern "C" fn() -> *mut dyn HaCK;


// pub struct DynamicHaC {
//     pub lib: Library,
//     pub instance: Rc<RefCell<dyn HaCK>>,
//     pub type_id: TypeId,
// }

// impl std::fmt::Debug for DynamicHaC {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "DynamicHaC")
//     }
// }
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
        self.hacs.insert(TypeId::of::<T>(), Rc::new(RefCell::new(module)));
        self.menu_dirty = true.into();
    }
    
    pub fn register_boxed(&mut self, module: Rc<RefCell<dyn HaCK>>) {
        let type_id = {
            let m_ref = module.borrow();
            m_ref.nac_type_id()
        };
        self.hacs.insert(type_id, module);
    }

    pub fn eject_module<T: HaCK + 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(module_rc) = self.hacs.remove(&type_id) {
            module_rc.borrow_mut().on_unload();
            self.menu_dirty = true.into();
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
        self.hacs.get(&TypeId::of::<T>()).map(|rc| {
            std::cell::Ref::map(rc.borrow(), |m| m.as_any().downcast_ref::<T>().unwrap())
        })
    }

    pub fn get_mut<T: HaCK + 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        self.hacs.get(&TypeId::of::<T>()).map(|rc| {
            std::cell::RefMut::map(rc.borrow_mut(), |m| m.as_any_mut().downcast_mut::<T>().unwrap())
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

    // /// Dynamically load a `.dll` / `.so` / `.dylib` module and register it.
    // pub unsafe fn load_dynamic<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
    //     let lib = Library::new(path.as_ref())?;

    //     #[allow(improper_ctypes_definitions)]
    //     type CreateHaCFn = unsafe extern "C" fn() -> *mut dyn HaCK;

    //     let constructor: Symbol<CreateHaCFn> = lib.get(b"create_nac")?;
    //     let raw = constructor();
    //     let boxed: Box<dyn HaCK> = Box::from_raw(raw);
    //     let type_id = boxed.nac_type_id();

    //     self.register_boxed(Rc::new(RefCell::new(boxed)));

    //     self.loaded_libs.push(DynamicHaC {
    //         lib,
    //         instance: type_id,
    //         type_id,
    //     });

    //     Ok(())
    // }

    // /// Unload a dynamically loaded HaC and remove it from the container.
    // pub unsafe fn unload_dynamic(&mut self, type_id: TypeId) -> Result<()> {
    //     if !self.eject_module_by_id(type_id) {
    //         anyhow::bail!("Module not found in container");
    //     }

    //     if let Some(pos) = self.loaded_libs.iter().position(|m| m.type_id == type_id) {
    //         let dynamic = self.loaded_libs.remove(pos);
    //         drop(dynamic);
    //         Ok(())
    //     } else {
    //         anyhow::bail!("No dynamic library entry found for module");
    //     }
    // }

    // pub fn reload_module<P: AsRef<Path>>(&mut self, path: P, type_id: TypeId) -> Result<()> {
    //     // unsafe {
    //     //     self.unload_dynamic(type_id)?;
    //     //     self.load_dynamic(path)
    //     // }
    // }
}
