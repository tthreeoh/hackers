use crate::{hack::HaCK, HaCKS};

pub trait ModuleIterable {
    fn for_each_module_mut<F: FnMut(&mut dyn HaCK)>(&mut self, f: F);
    fn for_each_module<F: FnMut(&dyn HaCK)>(&self, f: F);
}


impl ModuleIterable for HaCKS {
    fn for_each_module_mut<F: FnMut(&mut dyn HaCK)>(&mut self, mut f: F) {
        for module in self.hacs.values() {
            f(&mut *module.borrow_mut());
        }
    }

    fn for_each_module<F: FnMut(&dyn HaCK)>(&self, mut f: F) {
        for module in self.hacs.values() {
            f(&*module.borrow());
        }
    }
}

impl HaCKS {
    pub fn iter_modules(&self) -> impl Iterator<Item = std::cell::Ref<'_, dyn HaCK>> + '_ {
        self.hacs.values().map(|m| m.borrow())
    }

    pub fn iter_modules_mut(&self) -> impl Iterator<Item = std::cell::RefMut<'_, dyn HaCK>> + '_ {
        self.hacs.values().map(|m| m.borrow_mut())
    }

    pub fn for_each_module<F: FnMut(&dyn HaCK)>(&self, mut f: F) {
        for module in self.hacs.values() {
            f(&*module.borrow());
        }
    }

    pub fn for_each_module_mut<F: FnMut(&mut dyn HaCK)>(&self, mut f: F) {
        for module in self.hacs.values() {
            f(&mut *module.borrow_mut());
        }
    }
}