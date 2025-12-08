use std::{any::TypeId, collections::BTreeMap};

use crate::HaCKS;

pub struct MenuCache {
    pub top_level: BTreeMap<String, Vec<(Vec<String>, String)>>,
}

impl HaCKS {
    pub fn rebuild_menu_cache(&mut self) -> MenuCache {
        let mut menu_tree: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();

        let keys: Vec<_> = self.hacs.keys().cloned().collect();
        // Inline sort by menu weight
        let mut sorted = keys;
        sorted.sort_by(|a, b| {
            let wa = self
                .hacs
                .get(a)
                .map(|m| m.borrow().menu_weight())
                .unwrap_or(0.0);
            let wb = self
                .hacs
                .get(b)
                .map(|m| m.borrow().menu_weight())
                .unwrap_or(0.0);
            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
        });

        for name_key in sorted {
            if let Some(module_rc) = self.hacs.get(&name_key) {
                let module = module_rc.borrow();
                let path: Vec<String> = module.menu_path().iter().map(|s| s.to_string()).collect();
                menu_tree.entry(path).or_default().push(name_key);
            }
        }

        let mut top_level: BTreeMap<String, Vec<(Vec<String>, String)>> = BTreeMap::new();
        for (path, names) in menu_tree {
            if let Some(first) = path.first() {
                for name in names {
                    top_level
                        .entry(first.clone())
                        .or_default()
                        .push((path.clone(), name));
                }
            }
        }

        MenuCache { top_level }
    }

    pub fn find_entries_for_path(
        &self,
        cache: &MenuCache,
        target_path: &[String],
    ) -> Vec<(Vec<String>, String)> {
        if target_path.is_empty() {
            return vec![];
        }

        let top = &target_path[0];
        if let Some(entries) = cache.top_level.get(top) {
            if target_path.len() == 1 {
                return entries.clone();
            }

            // Filter entries that match the path
            entries
                .iter()
                .filter(|(entry_path, _)| entry_path.starts_with(target_path))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
}
