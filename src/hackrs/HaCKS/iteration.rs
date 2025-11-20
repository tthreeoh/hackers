use crate::{hack::HaCK, HaCKS};

pub trait ModuleIterable {
    fn for_each_module_mut<F: FnMut(&mut dyn HaCK)>(&mut self, f: F);
    fn for_each_module<F: FnMut(&dyn HaCK)>(&self, f: F);
}


impl ModuleIterable for HaCKS {
    fn for_each_module_mut<F: FnMut(&mut dyn HaCK)>(&mut self, mut f: F) {
        for module in self.hacs.values_mut() {
            f(module.as_mut());
        }
    }

    fn for_each_module<F: FnMut(&dyn HaCK)>(&self, mut f: F) {
        for module in self.hacs.values() {
            f(module.as_ref());
        }
    }
}

impl HaCKS {
    pub fn iter_modules(&self) -> impl Iterator<Item = &Box<dyn HaCK>> {
        self.hacs.values()
    }

    pub fn iter_modules_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn HaCK>> {
        self.hacs.values_mut()
    }

    pub fn for_each_module<F: FnMut(&dyn HaCK)>(&self, mut f: F) {
        for module in self.hacs.values() {
            f(module.as_ref());
        }
    }

    pub fn for_each_module_mut<F: FnMut(&mut dyn HaCK)>(&mut self, mut f: F) {
        for module in self.hacs.values_mut() {
            f(module.as_mut());
        }
    }


}