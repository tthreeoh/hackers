
use std::any::TypeId;
use std::collections::HashMap;
use std::cell::RefCell;

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
    #[allow(unused)]
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

pub struct SyncRegistryBuilder {
    registry: SyncRegistry,
}

impl SyncRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: SyncRegistry::new(),
        }
    }
    
    /// Fluent API for one-way sync
    pub fn sync<S, T>(mut self, sync_fn: impl Fn(&S, &mut T) + Send + 'static) -> Self
    where
        S: crate::HaCK + 'static,
        T: crate::HaCK + 'static,
    {
        self.registry.register_one_way(sync_fn);
        self
    }
    
    /// Fluent API for bidirectional sync
    pub fn sync_bidirectional<A, B>(
        mut self,
        a_to_b: impl Fn(&A, &mut B) + Send + 'static,
        b_to_a: impl Fn(&B, &mut A) + Send + 'static,
    ) -> Self
    where
        A: crate::HaCK + 'static,
        B: crate::HaCK + 'static,
    {
        self.registry.register_bidirectional(a_to_b, b_to_a);
        self
    }
    
    pub fn build(self) -> SyncRegistry {
        self.registry
    }
}





/// Tracks the last known state of a field to detect changes
pub struct ChangeTracker<T: Clone + PartialEq> {
    last_known: RefCell<Option<T>>,
}

impl<T: Clone + PartialEq> ChangeTracker<T> {
    pub fn new() -> Self {
        Self {
            last_known: RefCell::new(None),
        }
    }

    /// Check if value changed since last check
    pub fn did_change(&self, current: &T) -> bool {
        let changed = self.last_known.borrow()
            .as_ref()
            .map(|last| last != current)
            .unwrap_or(true); // First time = changed
        
        if changed {
            *self.last_known.borrow_mut() = Some(current.clone());
        }
        
        changed
    }

    /// Manually update the tracked value without triggering change
    pub fn update(&self, value: T) {
        *self.last_known.borrow_mut() = Some(value);
    }
}

// ============================================================
// Change-Aware Bidirectional Sync
// ============================================================

/// Simpler version using Clone + PartialEq fields directly
pub struct SimpleBiDirectionalSync<A, B, T, FA, FB, GA, GB>
where
    A: crate::HaCK + 'static,
    B: crate::HaCK + 'static,
    T: Clone + PartialEq + Send + 'static,
    FA: Fn(&A, &mut B, &T) + Send + Sync + 'static,
    FB: Fn(&B, &mut A, &T) + Send + Sync + 'static,
    GA: Fn(&A) -> T + Send + Sync + 'static,
    GB: Fn(&B) -> T + Send + Sync + 'static,
{
    a_to_b: FA,
    b_to_a: FB,
    get_a_value: GA,
    get_b_value: GB,
    last_a: RefCell<Option<T>>,
    last_b: RefCell<Option<T>>,
    _phantom: std::marker::PhantomData<(A, B)>,
}

impl<A, B, T, FA, FB, GA, GB> SimpleBiDirectionalSync<A, B, T, FA, FB, GA, GB>
where
    A: crate::HaCK + 'static,
    B: crate::HaCK + 'static,
    T: Clone + PartialEq + Send + 'static,
    FA: Fn(&A, &mut B, &T) + Send + Sync + 'static,
    FB: Fn(&B, &mut A, &T) + Send + Sync + 'static,
    GA: Fn(&A) -> T + Send + Sync + 'static,
    GB: Fn(&B) -> T + Send + Sync + 'static,
{
    pub fn new(a_to_b: FA, b_to_a: FB, get_a_value: GA, get_b_value: GB) -> Self {
        Self {
            a_to_b,
            b_to_a,
            get_a_value,
            get_b_value,
            last_a: RefCell::new(None),
            last_b: RefCell::new(None),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A, B, T, FA, FB, GA, GB> crate::hackrs::sync::SyncAction 
    for SimpleBiDirectionalSync<A, B, T, FA, FB, GA, GB>
where
    A: crate::HaCK + 'static,
    B: crate::HaCK + 'static,
    T: Clone + PartialEq + Send + 'static,
    FA: Fn(&A, &mut B, &T) + Send + Sync + 'static,
    FB: Fn(&B, &mut A, &T) + Send + Sync + 'static,
    GA: Fn(&A) -> T + Send + Sync + 'static,
    GB: Fn(&B) -> T + Send + Sync + 'static,
{
    fn apply(&self, hacs: &crate::HaCKS) {
        // Check A for changes
        let a_changed = if let Some(a) = hacs.get_module::<A>() {
            let current = (self.get_a_value)(&*a);
            let changed = self.last_a.borrow()
                .as_ref()
                .map(|last| last != &current)
                .unwrap_or(true);
            
            if changed {
                *self.last_a.borrow_mut() = Some(current.clone());
            }
            
            (changed, Some(current))
        } else {
            (false, None)
        };

        // Check B for changes
        let b_changed = if let Some(b) = hacs.get_module::<B>() {
            let current = (self.get_b_value)(&*b);
            let changed = self.last_b.borrow()
                .as_ref()
                .map(|last| last != &current)
                .unwrap_or(true);
            
            if changed {
                *self.last_b.borrow_mut() = Some(current.clone());
            }
            
            (changed, Some(current))
        } else {
            (false, None)
        };

        // Apply sync based on who changed
        match (a_changed, b_changed) {
            ((true, Some(a_val)), (false, _)) => {
                // A changed, update B
                if let Some(a) = hacs.get_module::<A>() {
                    if let Some(mut b) = hacs.get_module_mut::<B>() {
                        (self.a_to_b)(&*a, &mut *b, &a_val);
                        // Update B tracker
                        let new_b = (self.get_b_value)(&*b);
                        *self.last_b.borrow_mut() = Some(new_b);
                    }
                }
            }
            ((false, _), (true, Some(b_val))) => {
                // B changed, update A
                if let Some(b) = hacs.get_module::<B>() {
                    if let Some(mut a) = hacs.get_module_mut::<A>() {
                        (self.b_to_a)(&*b, &mut *a, &b_val);
                        // Update A tracker
                        let new_a = (self.get_a_value)(&*a);
                        *self.last_a.borrow_mut() = Some(new_a);
                    }
                }
            }
            ((true, Some(a_val)), (true, _)) => {
                // Both changed - A wins
                if let Some(a) = hacs.get_module::<A>() {
                    if let Some(mut b) = hacs.get_module_mut::<B>() {
                        (self.a_to_b)(&*a, &mut *b, &a_val);
                    }
                }
            }
            _ => {
                // No changes or no modules
            }
        }
    }
}

// ============================================================
// Builder Extension
// ============================================================

pub trait SmartSyncBuilder {
    /// Register a change-aware bidirectional sync
    fn sync_smart<A, B, T>(
        self,
        a_to_b: impl Fn(&A, &mut B, &T) + Send + Sync + 'static,
        b_to_a: impl Fn(&B, &mut A, &T) + Send + Sync + 'static,
        get_a: impl Fn(&A) -> T + Send + Sync + 'static,
        get_b: impl Fn(&B) -> T + Send + Sync + 'static,
    ) -> Self
    where
        A: crate::HaCK + 'static,
        B: crate::HaCK + 'static,
        T: Clone + PartialEq + Send + 'static;
}

impl SmartSyncBuilder for crate::hackrs::sync::SyncRegistryBuilder {
    fn sync_smart<A, B, T>(
        mut self,
        a_to_b: impl Fn(&A, &mut B, &T) + Send + Sync + 'static,
        b_to_a: impl Fn(&B, &mut A, &T) + Send + Sync + 'static,
        get_a: impl Fn(&A) -> T + Send + Sync + 'static,
        get_b: impl Fn(&B) -> T + Send + Sync + 'static,
    ) -> Self
    where
        A: crate::HaCK + 'static,
        B: crate::HaCK + 'static,
        T: Clone + PartialEq + Send + 'static,
    {
        // Access the internal registry and add the sync
        // This requires SyncRegistry to expose an add method
        // For now, we'll assume it's accessible via a method
        self.registry.actions.push(Box::new(SimpleBiDirectionalSync::new(
            a_to_b,
            b_to_a,
            get_a,
            get_b,
        )));
        self
    }
}

// ============================================================
// USAGE EXAMPLE
// ============================================================

/*
use hackers::sync::SmartSyncBuilder;

pub fn build_sync_registry() -> SyncRegistry {
    SyncRegistryBuilder::new()
        .sync_smart(
            // A -> B: When PanicButton changes, update Chicken
            |panic: &PanicButton, chicken: &mut Chicken, value: &(bool, f32)| {
                chicken.enabled = value.0;
                chicken.hp_percent = value.1;
            },
            // B -> A: When Chicken changes, update PanicButton
            |chicken: &Chicken, panic: &mut PanicButton, value: &(bool, f32)| {
                panic.emergency_mode = value.0;
                panic.emergency_threshold = value.1;
            },
            // Extract A's values to track
            |panic: &PanicButton| (panic.emergency_mode, panic.emergency_threshold),
            // Extract B's values to track
            |chicken: &Chicken| (chicken.enabled, chicken.hp_percent),
        )
        .build()
}
*/