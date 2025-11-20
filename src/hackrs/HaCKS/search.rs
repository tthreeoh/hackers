use std::{any::TypeId, collections::BTreeMap};

use crate::HaCKS;

pub struct MenuCache {
    pub top_level: BTreeMap<String, Vec<(Vec<String>, TypeId)>>,
}

impl HaCKS {
    pub fn rebuild_menu_cache(&mut self) -> MenuCache {
        let mut menu_tree: BTreeMap<Vec<String>, Vec<TypeId>> = BTreeMap::new();
        
        let type_ids: Vec<_> = self.hacs.keys().copied().collect();
        let sorted = self.sort_by_weight(type_ids, |m| m.menu_weight());  // USE MENU WEIGHT
        
        for type_id in sorted {
            if let Some(module) = self.hacs.get(&type_id) {
                let path: Vec<String> = module.menu_path().iter().map(|s| s.to_string()).collect();
                menu_tree.entry(path).or_default().push(type_id);
            }
        }
    
        let mut top_level: BTreeMap<String, Vec<(Vec<String>, TypeId)>> = BTreeMap::new();
        for (path, ids) in menu_tree {
            if let Some(first) = path.first() {
                for id in ids {
                    top_level.entry(first.clone()).or_default().push((path.clone(), id));
                }
            }
        }
    
        MenuCache { top_level }
    }

    pub fn find_entries_for_path(&self, cache: &MenuCache, target_path: &[String]) -> Vec<(Vec<String>, TypeId)> {
        if target_path.is_empty() {
            return vec![];
        }
        
        let top = &target_path[0];
        if let Some(entries) = cache.top_level.get(top) {
            if target_path.len() == 1 {
                return entries.clone();
            }
            
            // Filter entries that match the path
            entries.iter()
                .filter(|(entry_path, _)| entry_path.starts_with(target_path))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
   
}