
use std::any::TypeId;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A type-safe sync action between two modules
pub trait SyncAction: Send + 'static {
    fn apply(&self, hacs: &crate::HaCKS);
}

/// One-way sync: Source -> Target
pub struct OneWaySync<S, T, F>
where
    S: crate::HaCK + 'static,
    T: crate::HaCK + 'static,
    F: Fn(&S, &mut T) + Send + 'static,
{
    sync_fn: F,
    _phantom: std::marker::PhantomData<(S, T)>,
}

impl<S, T, F> OneWaySync<S, T, F>
where
    S: crate::HaCK + 'static,
    T: crate::HaCK + 'static,
    F: Fn(&S, &mut T) + Send + 'static,
{
    pub fn new(sync_fn: F) -> Self {
        Self {
            sync_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, T, F> SyncAction for OneWaySync<S, T, F>
where
    S: crate::HaCK + 'static,
    T: crate::HaCK + 'static,
    F: Fn(&S, &mut T) + Send + 'static,
{
    fn apply(&self, hacs: &crate::HaCKS) {
        if let Some(source) = hacs.get_module::<S>() {
            if let Some(mut target) = hacs.get_module_mut::<T>() {
                (self.sync_fn)(&*source, &mut *target);
            }
        }
    }
}

/// Bidirectional sync: Source <-> Target
pub struct BiDirectionalSync<A, B, FA, FB>
where
    A: crate::HaCK + 'static,
    B: crate::HaCK + 'static,
    FA: Fn(&A, &mut B) + Send + 'static,
    FB: Fn(&B, &mut A) + Send + 'static,
{
    a_to_b: FA,
    b_to_a: FB,
    _phantom: std::marker::PhantomData<(A, B)>,
}

impl<A, B, FA, FB> BiDirectionalSync<A, B, FA, FB>
where
    A: crate::HaCK + 'static,
    B: crate::HaCK + 'static,
    FA: Fn(&A, &mut B) + Send + 'static,
    FB: Fn(&B, &mut A) + Send + 'static,
{
    pub fn new(a_to_b: FA, b_to_a: FB) -> Self {
        Self {
            a_to_b,
            b_to_a,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A, B, FA, FB> SyncAction for BiDirectionalSync<A, B, FA, FB>
where
    A: crate::HaCK + 'static,
    B: crate::HaCK + 'static,
    FA: Fn(&A, &mut B) + Send + 'static,
    FB: Fn(&B, &mut A) + Send + 'static,
{
    fn apply(&self, hacs: &crate::HaCKS) {
        // Get both modules and detect who changed
        if let (Some(a), Some(b)) = (hacs.get_module::<A>(), hacs.get_module::<B>()) {
            // Clone values to detect changes (requires Clone on relevant fields)
            drop(a);
            drop(b);
            
            // Apply A -> B
            if let Some(a) = hacs.get_module::<A>() {
                if let Some(mut b) = hacs.get_module_mut::<B>() {
                    (self.a_to_b)(&*a, &mut *b);
                }
            }
            
            // Apply B -> A
            if let Some(b) = hacs.get_module::<B>() {
                if let Some(mut a) = hacs.get_module_mut::<A>() {
                    (self.b_to_a)(&*b, &mut *a);
                }
            }
        }
    }
}

/// Registry of all sync actions
#[derive(Default)]
pub struct SyncRegistry {
    actions: Vec<Box<dyn SyncAction>>,
    priority_map: HashMap<TypeId, i32>,
}

impl SyncRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a one-way sync: Source -> Target
    pub fn register_one_way<S, T, F>(&mut self, sync_fn: F)
    where
        S: crate::HaCK + 'static,
        T: crate::HaCK + 'static,
        F: Fn(&S, &mut T) + Send + 'static,
    {
        self.actions.push(Box::new(OneWaySync::new(sync_fn)));
    }
    
    /// Register a bidirectional sync: A <-> B
    pub fn register_bidirectional<A, B, FA, FB>(&mut self, a_to_b: FA, b_to_a: FB)
    where
        A: crate::HaCK + 'static,
        B: crate::HaCK + 'static,
        FA: Fn(&A, &mut B) + Send + 'static,
        FB: Fn(&B, &mut A) + Send + 'static,
    {
        self.actions.push(Box::new(BiDirectionalSync::new(a_to_b, b_to_a)));
    }
    
    /// Apply all registered syncs
    pub fn apply_all(&self, hacs: &crate::HaCKS) {
        for action in &self.actions {
            action.apply(hacs);
        }
    }
}