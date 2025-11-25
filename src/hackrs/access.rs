use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Defines what level of access a module grants to others
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// No access - module cannot be accessed by others
    None,
    /// Read-only access - others can borrow immutably
    ReadOnly,
    /// Full access - others can borrow mutably
    ReadWrite,
}

impl Default for AccessLevel {
    fn default() -> Self {
        AccessLevel::ReadOnly
    }
}

/// Access control configuration for a module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessControl {
    /// Default access level granted to all modules
    pub default_access: AccessLevel,
    /// Specific access overrides for certain modules (TypeId -> AccessLevel)
    #[serde(skip)]
    pub specific_access: HashMap<TypeId, AccessLevel>,
    /// Whitelist of modules that can access this one (empty = all allowed)
    #[serde(skip)]
    pub whitelist: HashSet<TypeId>,
    /// Blacklist of modules that cannot access this one
    #[serde(skip)]
    pub blacklist: HashSet<TypeId>,
}

impl Default for AccessControl {
    fn default() -> Self {
        Self {
            default_access: AccessLevel::ReadOnly,
            specific_access: HashMap::new(),
            whitelist: HashSet::new(),
            blacklist: HashSet::new(),
        }
    }
}

impl AccessControl {
    pub fn new(default: AccessLevel) -> Self {
        Self {
            default_access: default,
            ..Default::default()
        }
    }

    /// Set access level for a specific module type
    pub fn grant<T: 'static>(&mut self, level: AccessLevel) {
        self.specific_access.insert(TypeId::of::<T>(), level);
    }

    /// Add a module to the whitelist
    pub fn allow<T: 'static>(&mut self) {
        self.whitelist.insert(TypeId::of::<T>());
    }

    /// Add a module to the blacklist
    pub fn deny<T: 'static>(&mut self) {
        self.blacklist.insert(TypeId::of::<T>());
    }

    /// Check if a module can access at a given level
    pub fn can_access(&self, requester: TypeId, level: AccessLevel) -> bool {
        // Check blacklist first
        if self.blacklist.contains(&requester) {
            return false;
        }

        // Check whitelist if non-empty
        if !self.whitelist.is_empty() && !self.whitelist.contains(&requester) {
            return false;
        }

        // Get the granted access level
        let granted = self.specific_access
            .get(&requester)
            .copied()
            .unwrap_or(self.default_access);

        // Check if granted level is sufficient
        match (granted, level) {
            (AccessLevel::None, _) => false,
            (AccessLevel::ReadOnly, AccessLevel::None) => true,
            (AccessLevel::ReadOnly, AccessLevel::ReadOnly) => true,
            (AccessLevel::ReadOnly, AccessLevel::ReadWrite) => false,
            (AccessLevel::ReadWrite, _) => true,
        }
    }
}

/// Runtime access override system
#[derive(Debug, Default)]
pub struct AccessManager {
    /// Global overrides that bypass module-level access control
    global_overrides: HashMap<TypeId, AccessLevel>,
    /// Emergency override - grants full access to everything
    emergency_override: bool,
}

impl AccessManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a global override for a specific module
    pub fn set_override<T: 'static>(&mut self, level: AccessLevel) {
        self.global_overrides.insert(TypeId::of::<T>(), level);
    }

    /// Remove a global override
    pub fn clear_override<T: 'static>(&mut self) {
        self.global_overrides.remove(&TypeId::of::<T>());
    }

    /// Enable emergency override (grants full access everywhere)
    pub fn enable_emergency(&mut self) {
        self.emergency_override = true;
    }

    /// Disable emergency override
    pub fn disable_emergency(&mut self) {
        self.emergency_override = false;
    }

    /// Check if access is allowed, considering overrides
    pub fn check_access(
        &self,
        target: TypeId,
        requester: TypeId,
        level: AccessLevel,
        module_acl: &AccessControl,
    ) -> bool {
        // Emergency override bypasses everything
        if self.emergency_override {
            return true;
        }

        // Check global overrides
        if let Some(override_level) = self.global_overrides.get(&target) {
            return match (*override_level, level) {
                (AccessLevel::None, _) => false,
                (AccessLevel::ReadOnly, AccessLevel::ReadWrite) => false,
                _ => true,
            };
        }

        // Fall back to module-level ACL
        module_acl.can_access(requester, level)
    }
}

/// Token proving access rights
pub struct AccessToken {
    pub requester: TypeId,
    pub target: TypeId,
    pub level: AccessLevel,
}

impl AccessToken {
    pub fn can_write(&self) -> bool {
        matches!(self.level, AccessLevel::ReadWrite)
    }
}

// Add to HaCMetadata
use crate::metadata::HaCMetadata;

impl HaCMetadata {
    /// Get the access control for this module
    pub fn access_control(&self) -> &AccessControl {
        &self.access_control
    }

    /// Get mutable access to configure access control
    pub fn access_control_mut(&mut self) -> &mut AccessControl {
        &mut self.access_control
    }
}