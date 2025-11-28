use std::cell::{Ref, RefCell, RefMut};
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
#[allow(unused)]
pub use registry::*;
pub use iteration::*;
pub use search::*;
pub use events::*;

use crate::{GlobalStateTracker, RuntimeSyncManager, SyncRegistry};
use crate::access::AccessManager;
use crate::gui::hotkey_manager::HotkeyManager;

use crate::hack::HaCK;
#[allow(unused)]
pub struct HaCKS {
    // Event bus needs RefCell for interior mutability
    pub event_bus: RefCell<Vec<HaCSEvent>>,
    
    pub hacs: HashMap<TypeId, Rc<RefCell<dyn HaCK>>>,
    pub init_data: HashMap<TypeId, Box<dyn Any + Send>>,
    
    // These need RefCell if modified during &self methods
    pub menu_dirty: RefCell<bool>,
    pub menu_cache: RefCell<Option<MenuCache>>,
    
    pub hotkey_manager: RefCell<HotkeyManager>,
    pub triggered_hotkeys: RefCell<Vec<String>>,
    
    pub show_debug_window: RefCell<bool>,
    pub windowed_groups: RefCell<HashMap<Vec<String>, bool>>,
    pub metadata_window: RefCell<bool>,
    pub viz_mode: RefCell<u32>,
    pub metadata_window_viz: RefCell<bool>,
    pub color_scheme: RefCell<usize>,
    
    pub access_manager: RefCell<AccessManager>,
    pub sync_registry: RefCell<Option<SyncRegistry>>,
    pub runtime_sync_manager: RefCell<Option<RuntimeSyncManager>>,
    pub state_tracker: RefCell<GlobalStateTracker>,
}

#[allow(unused)]
impl HaCKS {
    pub fn new() -> Self {
        HaCKS {
            event_bus: RefCell::new(Vec::new()),
            hacs: HashMap::new(),
            init_data: HashMap::new(),
            menu_cache: RefCell::new(None),
            menu_dirty: RefCell::new(false),
            hotkey_manager: RefCell::new(HotkeyManager::new()),
            triggered_hotkeys: RefCell::new(Vec::new()),
            show_debug_window: RefCell::new(false),
            metadata_window: RefCell::new(false),
            windowed_groups: RefCell::new(HashMap::new()),
            viz_mode: RefCell::new(0),
            metadata_window_viz: RefCell::new(false),
            color_scheme: RefCell::new(0),
            access_manager: RefCell::new(AccessManager::new()),
            sync_registry: RefCell::new(None),
            runtime_sync_manager: RefCell::new(None),
            state_tracker: RefCell::new(GlobalStateTracker::new()),
        }
    }

    pub fn toggle_state_tracker(&self) {
        let mut tracker = self.state_tracker.borrow_mut();
        tracker.show_window = !tracker.show_window;
    }

    pub fn show_state_tracker(&self) {
        self.state_tracker.borrow_mut().show_window = true;
    }

    pub fn init_sync_registry(&self, registry: SyncRegistry) {
        *self.sync_registry.borrow_mut() = Some(registry);
    }
    
    pub fn init_runtime_sync_manager(&self, manager: RuntimeSyncManager) {
        *self.runtime_sync_manager.borrow_mut() = Some(manager);
    }
    
    /// Run all syncs
    pub fn sync_modules(&self) {
        // Run type-safe syncs
        if let Some(registry) = self.sync_registry.borrow().as_ref() {
            registry.apply_all(self);
        }
        
        // Run runtime syncs
        if let Some(manager) = self.runtime_sync_manager.borrow_mut().as_mut() {
            manager.apply_all(self);
        }
    }

    // pub fn get_module<T: HaCK + 'static>(&self) -> Option<std::cell::Ref<'_, T>> {
    //     self.hacs.get(&TypeId::of::<T>()).map(|rc| {
    //         std::cell::Ref::map(rc.borrow(), |m| {
    //             m.as_any().downcast_ref::<T>().unwrap()
    //         })
    //     })
    // }

    // pub fn get_module_mut<T: HaCK + 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
    //     self.hacs.get(&TypeId::of::<T>()).map(|rc| {
    //         std::cell::RefMut::map(rc.borrow_mut(), |m| {
    //             m.as_any_mut().downcast_mut::<T>().unwrap()
    //         })
    //     })
    // }

    pub fn get_module<T: HaCK + 'static>(&self) -> Option<std::cell::Ref<'_, T>> {
        let type_id = std::any::TypeId::of::<T>();
        self.hacs.get(&type_id).map(|rc| {
            std::cell::Ref::map(rc.borrow(), |m| {
                m.as_any()
                    .downcast_ref::<T>()
                    .expect("TypeId matched but downcast failed")
            })
        })
    }

    pub fn get_module_mut<T: HaCK + 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        let type_id = std::any::TypeId::of::<T>();
        self.hacs.get(&type_id).map(|rc| {
            std::cell::RefMut::map(rc.borrow_mut(), |m| {
                m.as_any_mut()
                    .downcast_mut::<T>()
                    .expect("TypeId matched but downcast failed")
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

pub struct ModuleAccess<'a> {
    hacs: &'a mut HaCKS,
    self_id: TypeId,
}

impl<'a> ModuleAccess<'a> {
    
    pub fn new(hacs: &'a mut HaCKS, self_id: TypeId) -> Self {
        Self { hacs, self_id }
    }

    pub fn get_mut<T: HaCK + 'static>(&self) -> Option<RefMut<'_, T>> {
        let id = TypeId::of::<T>();
        if id == self.self_id {
            return None; // you can't borrow yourself
        }

        self.hacs.get_module_mut::<T>()
    }

    pub fn get<T: HaCK + 'static>(&self) -> Option<Ref<'_, T>> {
        let id = TypeId::of::<T>();
        if id == self.self_id {
            return None;
        }

        self.hacs.get_module::<T>()
    }
}