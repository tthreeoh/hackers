use std::cell::RefCell;
#[allow(unused)]
use std::rc::Rc;
#[allow(unused)]
use libloading::Library;
use std::any::{Any, TypeId};
use std::collections::HashMap;
pub mod sorting;
pub mod update;
pub mod persistence;
pub mod registry;
pub mod lifecycle;
pub mod iteration;
pub mod search;
pub mod events;


pub use update::*;
pub use registry::*;
pub use iteration::*;
pub use search::*;
pub use events::*;
use crate::gui::hotkey_manager::HotkeyManager;

use crate::hack::HaCK;
#[allow(unused)]
pub struct HaCKS {
    pub event_bus: Vec<HaCSEvent>,
    pub hacs: HashMap<TypeId, Rc<RefCell<dyn HaCK>>>,
    // pub loaded_libs: Vec<DynamicHaC>,
    pub init_data: HashMap<TypeId, Box<dyn Any + Send>>,
    pub menu_dirty: bool,
    pub menu_cache: Option<MenuCache>,
    
    pub hotkey_manager: HotkeyManager,
    pub triggered_hotkeys: Vec<String>,
    //gui
    pub show_debug_window: bool,
    pub windowed_groups: HashMap<Vec<String>, bool>,
    pub metadata_window: bool,
    pub viz_mode: u32,
    pub metadata_window_viz: bool,
    pub color_scheme: usize,
}

#[allow(unused)]
impl HaCKS {
    pub fn new() -> Self {
        HaCKS {
            event_bus: Vec::new(),
            hacs: HashMap::new(),
            // loaded_libs: Vec::new(),
            init_data: HashMap::new(),
            menu_cache: None,
            menu_dirty: false,

            hotkey_manager: HotkeyManager::new(),
            triggered_hotkeys: Vec::new(),
            show_debug_window: false,
            metadata_window: false,
            windowed_groups: Default::default(),
            viz_mode:Default::default(),
            metadata_window_viz: false,
            color_scheme: 0,
        }
    }

    pub fn get_module<T: HaCK + 'static>(&self) -> Option<std::cell::Ref<'_, T>> {
        self.hacs.get(&TypeId::of::<T>()).map(|rc| {
            std::cell::Ref::map(rc.borrow(), |m| {
                m.as_any().downcast_ref::<T>().unwrap()
            })
        })
    }

    pub fn get_module_mut<T: HaCK + 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        self.hacs.get(&TypeId::of::<T>()).map(|rc| {
            std::cell::RefMut::map(rc.borrow_mut(), |m| {
                m.as_any_mut().downcast_mut::<T>().unwrap()
            })
        })
    }

    pub fn get_state<T: HaCK + 'static, R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.get_module::<T>().map(|m| f(&*m))
        }
        
        
        /// Mutably operate on a module safely.
        pub fn with_module_mut<T: HaCK + 'static, F: FnOnce(&mut T)>(&self, f: F) {
            if let Some(mut m) = self.get_module_mut::<T>() {
                f(&mut *m);
            }
        }
}