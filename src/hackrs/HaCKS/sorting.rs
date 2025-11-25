use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{hack::HaCK, HaCKS};

impl HaCKS {

    pub fn sort_by_weight<F>(&self, type_ids: Vec<TypeId>, weight_fn: F) -> Vec<TypeId>
    where
        F: Fn(&Rc<RefCell<dyn HaCK>>) -> f32,
    {
        let mut weighted: Vec<_> = type_ids
            .into_iter()
            .filter_map(|id| self.hacs.get(&id).map(|m| (id, weight_fn(m))))
            .collect();
    
        weighted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        weighted.into_iter().map(|(id, _)| id).collect()
    }
    
    pub fn topological_sort_update(&self) -> Vec<TypeId> {
        let mut sorted = vec![];
        let mut visited = std::collections::HashSet::new();
    
        for module in self.hacs.values() {
            self.visit(module.borrow().nac_type_id(), &mut visited, &mut sorted);
        }
    
        sorted
    }
    
    pub fn visit(
        &self,
        type_id: TypeId,
        visited: &mut std::collections::HashSet<TypeId>,
        sorted: &mut Vec<TypeId>,
    ) {
        if visited.contains(&type_id) {
            return;
        }
        visited.insert(type_id);
    
        if let Some(module_rc) = self.hacs.get(&type_id) {
            let module = module_rc.borrow();
            for dep in module.update_dependencies() {
                self.visit(dep, visited, sorted);
            }
        }
    
        sorted.push(type_id);
    }
    
}
    