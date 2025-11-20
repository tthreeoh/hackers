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

use crate::hack::HaCK;
#[allow(unused)]
pub struct HaCKS {
    pub event_bus: Vec<HaCSEvent>,
    pub hacs: HashMap<TypeId, Box<dyn HaCK>>,
    pub loaded_libs: Vec<DynamicHaC>,
    pub init_data: HashMap<TypeId, Box<dyn Any + Send>>,
    pub menu_dirty: bool,
    pub menu_cache: Option<MenuCache>,
    
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
            loaded_libs: Vec::new(),
            init_data: HashMap::new(),
            menu_cache: None,
            menu_dirty: false,
            show_debug_window: false,
            metadata_window: false,
            windowed_groups: Default::default(),
            viz_mode:Default::default(),
            metadata_window_viz: false,
            color_scheme: 0,
        }
    }

    pub fn get_module<T: HaCK + 'static>(&self) -> Option<&T> {
        self.hacs.get(&TypeId::of::<T>())?
            .as_ref()
            .as_any()
            .downcast_ref::<T>()
    }

    /// Mutable reference to a module of type `T`.
    pub fn get_module_mut<T: HaCK + 'static>(&mut self) -> Option<&mut T> {
        self.hacs.get_mut(&TypeId::of::<T>())?
            .as_mut()
            .as_any_mut()
            .downcast_mut::<T>()
    }

    /// Extract a specific piece of readonly state from a module.
    pub fn get_state<T: HaCK + 'static, R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.get_module::<T>().map(f)
    }

    /// Mutably operate on a module safely.
    pub fn with_module_mut<T: HaCK + 'static, F: FnOnce(&mut T)>(&mut self, f: F) {
        if let Some(m) = self.get_module_mut::<T>() {
            f(m);
        }
    }
}