use std::any::TypeId;
use std::cell::{Ref, RefMut};
use crate::{HaCK, HaCKS};
use super::access::{AccessLevel, AccessToken};

impl HaCKS {
    /// Try to get immutable access to a module
    pub fn try_access<T: HaCK + 'static>(&self, requester: TypeId) -> Option<Ref<'_, T>> {
        let target_id = TypeId::of::<T>();
        
        // Get the module to check its ACL
        let module_rc = self.hacs.get(&target_id)?;
        let module = module_rc.borrow();
        let acl = module.metadata().access_control();
        
        // Check access (borrow access_manager through RefCell)
        let access_granted = self.access_manager.borrow().check_access(
            target_id,
            requester,
            AccessLevel::ReadOnly,
            acl,
        );
        
        if !access_granted {
            return None;
        }
        
        drop(module); // Release borrow
        
        // Return the reference
        self.get_module::<T>()
    }

    /// Try to get mutable access to a module
    pub fn try_access_mut<T: HaCK + 'static>(&self, requester: TypeId) -> Option<RefMut<'_, T>> {
        let target_id = TypeId::of::<T>();
        
        // Get the module to check its ACL
        let module_rc = self.hacs.get(&target_id)?;
        let module = module_rc.borrow();
        let acl = module.metadata().access_control();
        
        // Check access (borrow access_manager through RefCell)
        let access_granted = self.access_manager.borrow().check_access(
            target_id,
            requester,
            AccessLevel::ReadWrite,
            acl,
        );
        
        if !access_granted {
            return None;
        }
        
        drop(module); // Release borrow
        
        // Return the mutable reference
        self.get_module_mut::<T>()
    }

    /// Create an access token for later use
    pub fn request_access<T: HaCK + 'static>(
        &self,
        requester: TypeId,
        level: AccessLevel,
    ) -> Option<AccessToken> {
        let target_id = TypeId::of::<T>();
        let module_rc = self.hacs.get(&target_id)?;
        let module = module_rc.borrow();
        let acl = module.metadata().access_control();
        
        if self.access_manager.borrow().check_access(target_id, requester, level, acl) {
            Some(AccessToken {
                requester,
                target: target_id,
                level,
            })
        } else {
            None
        }
    }

    /// Execute a function with access to another module (immutable)
    pub fn with_access<T, R, F>(
        &self,
        requester: TypeId,
        f: F,
    ) -> Option<R>
    where
        T: HaCK + 'static,
        F: FnOnce(&T) -> R,
    {
        self.try_access::<T>(requester).map(|m| f(&*m))
    }

    /// Execute a function with mutable access to another module
    pub fn with_access_mut<T, R, F>(
        &self,
        requester: TypeId,
        f: F,
    ) -> Option<R>
    where
        T: HaCK + 'static,
        F: FnOnce(&mut T) -> R,
    {
        self.try_access_mut::<T>(requester).map(|mut m| f(&mut *m))
    }

    /// Access the global access manager (immutable)
    pub fn access_manager(&self) -> std::cell::Ref<'_, super::access::AccessManager> {
        self.access_manager.borrow()
    }

    /// Mutably access the global access manager
    pub fn access_manager_mut(&self) -> std::cell::RefMut<'_, super::access::AccessManager> {
        self.access_manager.borrow_mut()
    }
}

// Helper trait for modules to use
pub trait ModuleAccess {
    /// Get the module's TypeId for access requests
    fn type_id(&self) -> TypeId;

    /// Try to access another module immutably
    /// The lifetime of the returned Ref is tied to the HaCKS parameter
    fn access<'a, T: HaCK + 'static>(&self, hacs: &'a HaCKS) -> Option<Ref<'a, T>> {
        hacs.try_access::<T>(self.type_id())
    }

    /// Try to access another module mutably
    /// The lifetime of the returned RefMut is tied to the HaCKS parameter
    fn access_mut<'a, T: HaCK + 'static>(&self, hacs: &'a HaCKS) -> Option<RefMut<'a, T>> {
        hacs.try_access_mut::<T>(self.type_id())
    }

    /// Execute a function with access to another module
    fn with_access<T, R, F>(&self, hacs: &HaCKS, f: F) -> Option<R>
    where
        T: HaCK + 'static,
        F: FnOnce(&T) -> R,
    {
        hacs.with_access::<T, R, F>(self.type_id(), f)
    }

    /// Execute a function with mutable access to another module
    fn with_access_mut<T, R, F>(&self, hacs: &HaCKS, f: F) -> Option<R>
    where
        T: HaCK + 'static,
        F: FnOnce(&mut T) -> R,
    {
        hacs.with_access_mut::<T, R, F>(self.type_id(), f)
    }
}

// Auto-implement for all HaCK types
impl<H: HaCK> ModuleAccess for H {
    fn type_id(&self) -> TypeId {
        self.nac_type_id()
    }
}