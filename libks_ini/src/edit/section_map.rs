use std::collections::HashMap;

use crate::edit::Section;

#[derive(Debug, Clone)]
pub struct SectionMap {
    pub(crate) map: HashMap<String, Vec<usize>>,
    pub(crate) is_dirty: bool,
    pub(crate) is_enabled: bool,
}

impl SectionMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            is_dirty: false,
            is_enabled: true,
        }
    }
    
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&[usize]> {
        assert!(!self.is_dirty);
        self.map.get(key.as_ref())
            .map(Vec::as_slice)
    }
    
    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        assert!(!self.is_dirty);
        self.map.contains_key(key.as_ref())
    }
    
    pub fn rebuild(&mut self, sections: &[Section], source: &str) {
        if let Some(delta) = sections.len().checked_sub(self.map.capacity()) {
            self.map.reserve(delta);
        }
        
        for (i, section) in sections.iter().enumerate() {
            let key = section.header.key.to_str(source);
            self.map.entry(key.to_ascii_lowercase())
                .or_insert_with(|| Vec::new())
                .push(i);
        }
        
        self.is_dirty = false;
    }
    
    pub fn update_after_append<K: AsRef<str>>(&mut self, key: K, index: usize) {
        if !self.is_enabled {
            self.is_dirty = true;
            return;
        }
        
        self.map.entry(key.as_ref().to_ascii_lowercase())
            .and_modify(|indices| indices.push(index))
            .or_insert_with(|| vec![index]);
    }
    
    pub fn update_after_insert<K: AsRef<str>>(&mut self, key: K, index: usize) {
        if !self.is_enabled {
            self.is_dirty = true;
            return;
        }
        
        // All indices after the insert index shift forward 1
        for (_, indices) in self.map.iter_mut() {
            for i in indices.iter_mut().rev() {
                if *i < index {
                    break;
                }
                *i += 1;
            }
        }
        
        // Add the new index to the map
        self.map.entry(key.as_ref().to_ascii_lowercase())
            .and_modify(|indices| {
                // Insert in sorted order
                match indices.iter().position(|i| *i > index) {
                    Some(i) => indices.insert(i, index),
                    None => indices.push(index),
                }
            })
            .or_insert_with(|| vec![index]);
    }
    
    pub fn update_after_remove<K: AsRef<str>>(&mut self, key: K, index: usize) {
        if !self.is_enabled {
            self.is_dirty = true;
            return;
        }
        
        // Remove the old index from the map
        let key_lower = key.as_ref().to_ascii_lowercase();
        let indices = self.map.get_mut(&key_lower)
            .expect("Error updating section map (remove): key not found");
        let j = indices.iter().position(|j| *j == index)
            .expect("Error updating section map (remove): index not found");
        indices.remove(j);
        if indices.is_empty() {
            self.map.remove(&key_lower);
        }
        
        // All indices after the insert index shift backward 1
        for (_, indices) in self.map.iter_mut() {
            for i in indices.iter_mut().rev() {
                if *i < index {
                    break;
                }
                *i -= 1;
            }
        }
    }
    
    pub fn update_after_rename<K1, K2>(&mut self, index: usize, key_from: K1, key_to: K2)
    where
        K1: AsRef<str>,
        K2: AsRef<str>
    {
        if !self.is_enabled {
            self.is_dirty = true;
            return;
        }
        
        // Delete the index under the old key
        let key_from_lower = key_from.as_ref().to_ascii_lowercase();
        let indices = self.map.get_mut(&key_from_lower)
            .expect("Error updating section map (rename): key not found");
        let j = indices.iter().position(|j| *j == index)
            .expect("Error updating section map (rename): index not found");
        indices.remove(j);
        if indices.is_empty() {
            self.map.remove(&key_from_lower);
        }
        
        // Add the index under the new key
        let key_to_lower = key_to.as_ref().to_ascii_lowercase();
        self.map.entry(key_to_lower)
            .and_modify(|indices| {
                insert_in_order(indices, index)
            })
            .or_insert_with(|| vec![index]);
    }
    
    pub fn clear(&mut self) {
        self.map.clear();
        self.is_dirty = false;
    }
}

fn insert_in_order(indices: &mut Vec<usize>, i: usize) {
    match indices.iter().position(|j| *j > i) {
        Some(j) => indices.insert(j, i),
        None => indices.push(i),
    }
}
