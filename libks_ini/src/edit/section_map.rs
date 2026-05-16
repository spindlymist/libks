use std::collections::HashMap;

use crate::edit::Section;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionMap {
    pub(crate) map: HashMap<String, Vec<usize>>,
}

impl SectionMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&[usize]> {
        self.map.get(&key.as_ref().to_ascii_lowercase())
            .map(Vec::as_slice)
    }
    
    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.map.contains_key(&key.as_ref().to_ascii_lowercase())
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
    }
    
    pub fn update_after_append<K: AsRef<str>>(&mut self, key: K, index: usize) {
        self.map.entry(key.as_ref().to_ascii_lowercase())
            .and_modify(|indices| indices.push(index))
            .or_insert_with(|| vec![index]);
    }
    
    pub fn update_after_insert<K: AsRef<str>>(&mut self, key: K, index: usize) {
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
    }
    
    #[cfg(test)]
    pub fn ordering(&self) -> Vec<String> {
        let n_sections = self.map.values()
            .map(|indices| indices.len())
            .sum();
        
        let mut ordering: Vec<Option<String>> = vec![None; n_sections];
        for (key, indices) in &self.map {
            for &i in indices {
                assert!(ordering[i].is_none());
                ordering[i] = Some(key.clone());
            }
        }
        
        ordering.into_iter()
            .map(Option::unwrap)
            .collect()
    }
}

fn insert_in_order(indices: &mut Vec<usize>, i: usize) {
    match indices.iter().position(|j| *j > i) {
        Some(j) => indices.insert(j, i),
        None => indices.push(i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parser, item::Item, test_macros::*};
    
    macro_rules! expect {
        ($map:expr, $expected:expr) => {
            let mut items: Vec<_> = $map.map.into_iter()
                .collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            for (i, (key, indices)) in items.into_iter().enumerate() {
                assert_eq!(key, $expected[i].0);
                assert_eq!(indices, $expected[i].1);
            }
        }
    }
    
    const DUPLICATES: &'static str = before!("duplicates.ini");
    
    fn load_from_source(source: &str) -> (SectionMap, Vec<Section>) {
        let sections: Vec<_> = Parser::new(source)
            .filter_map(|item| match item {
                Item::Section(header) => Some(Section::from_header(header)),
                _ => None,
            })
            .collect();
        let mut map = SectionMap::new();
        map.rebuild(&sections, source);
        (map, sections)
    }
    
    #[test]
    fn rebuild_works() {
        let (map, _) = load_from_source(DUPLICATES);
        expect!(map, [
            ("section 0", [0, 3, 6]),
            ("section 1", [1, 4, 7]),
            ("section 2", [2, 5, 8]),
        ]);
    }
    
    #[test]
    fn get_works() {
        let (map, _) = load_from_source(DUPLICATES);
        assert_eq!(map.get("Section 0"), Some([0, 3, 6].as_slice()));
        assert_eq!(map.get("secTION 0"), Some([0, 3, 6].as_slice()));
        assert_eq!(map.get("section 4"), None);
    }
    
    #[test]
    fn has_works() {
        let (map, _) = load_from_source(DUPLICATES);
        assert_eq!(map.has("Section 0"), true);
        assert_eq!(map.has("secTION 0"), true);
        assert_eq!(map.has("section 4"), false);
    }
    
    #[test]
    fn update_after_append_works() {
        let (map, sections) = load_from_source(DUPLICATES);
        // Append new section
        {
            let mut map = map.clone();
            map.update_after_append("section 4", sections.len());
            expect!(map, [
                ("section 0", vec![0, 3, 6]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
                ("section 4", vec![9]),
            ]);
        }
        // Append existing section
        {
            let mut map = map.clone();
            map.update_after_append("section 0", sections.len());
            expect!(map.clone(), [
                ("section 0", vec![0, 3, 6, 9]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
            ]);
        }
    }
    
    #[test]
    fn update_after_insert_works() {
        let (map, sections) = load_from_source(DUPLICATES);
        // Insert new section, start
        {
            let mut map = map.clone();
            map.update_after_insert("section 4", 0);
            expect!(map, [
                ("section 0", vec![1, 4, 7]),
                ("section 1", vec![2, 5, 8]),
                ("section 2", vec![3, 6, 9]),
                ("section 4", vec![0]),
            ]);
        }
        // Insert new section, middle
        {
            let mut map = map.clone();
            map.update_after_insert("section 4", 5);
            expect!(map, [
                ("section 0", vec![0, 3, 7]),
                ("section 1", vec![1, 4, 8]),
                ("section 2", vec![2, 6, 9]),
                ("section 4", vec![5]),
            ]);
        }
        // Insert new section, end
        {
            let mut map = map.clone();
            map.update_after_insert("section 4", sections.len());
            expect!(map, [
                ("section 0", vec![0, 3, 6]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
                ("section 4", vec![9]),
            ]);
        }
        // Insert existing section, first of its name
        {
            let mut map = map.clone();
            map.update_after_insert("section 0", 0);
            expect!(map, [
                ("section 0", vec![0, 1, 4, 7]),
                ("section 1", vec![2, 5, 8]),
                ("section 2", vec![3, 6, 9]),
            ]);
        }
        // Insert existing section, middle of its name
        {
            let mut map = map.clone();
            map.update_after_insert("section 0", 5);
            expect!(map, [
                ("section 0", vec![0, 3, 5, 7]),
                ("section 1", vec![1, 4, 8]),
                ("section 2", vec![2, 6, 9]),
            ]);
        }
        // Insert existing section, last of its name
        {
            let mut map = map.clone();
            map.update_after_insert("section 0", sections.len());
            expect!(map, [
                ("section 0", vec![0, 3, 6, 9]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
            ]);
        }
    }
    
    #[test]
    fn update_after_remove_works() {
        let (map, _) = load_from_source(DUPLICATES);
        // Delete section, first of its name
        {
            let mut map = map.clone();
            map.update_after_remove("section 0", 0);
            expect!(map, [
                ("section 0", vec![2, 5]),
                ("section 1", vec![0, 3, 6]),
                ("section 2", vec![1, 4, 7]),
            ]);
        }
        // Delete section, middle of its name
        {
            let mut map = map.clone();
            map.update_after_remove("section 0", 3);
            expect!(map, [
                ("section 0", vec![0, 5]),
                ("section 1", vec![1, 3, 6]),
                ("section 2", vec![2, 4, 7]),
            ]);
        }
        // Delete section, last of its name
        {
            let mut map = map.clone();
            map.update_after_remove("section 0", 6);
            expect!(map, [
                ("section 0", vec![0, 3]),
                ("section 1", vec![1, 4, 6]),
                ("section 2", vec![2, 5, 7]),
            ]);
        }
        // If the section was unique, the key is removed from the map
        {
            let mut map = map.clone();
            map.update_after_remove("section 0", 6);
            map.update_after_remove("section 0", 3);
            map.update_after_remove("section 0", 0);
            expect!(map, [
                ("section 1", vec![0, 2, 4]),
                ("section 2", vec![1, 3, 5]),
            ]);
        }
    }
    
    #[test]
    fn update_after_rename_works() {
        let (map, sections) = load_from_source(DUPLICATES);
        // Renamed section is new
        {
            let mut map = map.clone();
            map.update_after_rename(3, "section 0", "section 4");
            expect!(map, [
                ("section 0", vec![0, 6]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
                ("section 4", vec![3]),
            ]);
        }
        // Renamed section merges with existing section
        {
            let mut map = map.clone();
            map.update_after_rename(3, "section 0", "section 1");
            expect!(map, [
                ("section 0", vec![0, 6]),
                ("section 1", vec![1, 3, 4, 7]),
                ("section 2", vec![2, 5, 8]),
            ]);
        }
        // If the section was unique, the old key is removed from the map
        {
            let mut map = map.clone();
            map.update_after_append("section 4", sections.len());
            expect!(map.clone(), [
                ("section 0", vec![0, 3, 6]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
                ("section 4", vec![9]),
            ]);
            
            map.update_after_rename(sections.len(), "section 4", "section 5");
            expect!(map, [
                ("section 0", vec![0, 3, 6]),
                ("section 1", vec![1, 4, 7]),
                ("section 2", vec![2, 5, 8]),
                ("section 5", vec![9]),
            ]);
        }
    }
}
