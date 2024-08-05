use std::{
    collections::HashMap,
    rc::Rc,
};

use crate::{
    item::Item,
    section::{
        Section,
        VirtualSection as VSection,
        VirtualSectionMut as VSectionMut,
    },
};

pub struct Ini {
    source: Rc<str>,
    global_section: Section,
    sections: Vec<Section>,
    section_index: HashMap<String, Vec<usize>>,
}

impl Ini {
    pub fn new(source: &str) -> Self {
        let source = Rc::<str>::from(source);
        let mut global_section = Section::new_global(Rc::clone(&source));
        let mut sections = Vec::new();

        for item in crate::parse::Parser::new(&source).map(Item::from) {
            match item {
                Item::Section(key, padding) => {
                    let header = Item::Section(key, padding);
                    let section = Section::new(Rc::clone(&source), header);
                    sections.push(section);
                },
                _ => match sections.last_mut() {
                    Some(section) => section.push_item(item),
                    None => global_section.push_item(item),
                },
            }
        }

        let section_index = Self::build_section_index(&sections);

        Self {
            source,
            global_section,
            sections,
            section_index,
        }
    }

    fn build_section_index(sections: &[Section]) -> HashMap<String, Vec<usize>> {
        let mut index = HashMap::<_, Vec<_>>::new();

        for (i, section) in sections.iter().enumerate() {
            let lower_key = section.key().to_ascii_lowercase();
            index.entry(lower_key)
                .and_modify(|indices| indices.push(i))
                .or_insert_with(|| vec![i]);
        }

        index
    }

    fn v_section<'a>(&'a self, indices: &[usize]) -> VSection<'a> {
        let sections = borrow_indices(&self.sections, indices);
        VSection::new(sections)
    }

    fn v_section_mut<'a>(&'a mut self, indices: &[usize]) -> VSectionMut<'a> {
        let sections = borrow_indices_mut(&mut self.sections, indices);
        VSectionMut::new(sections)
    }

    pub fn has_section(&self, key: &str) -> bool {
        self.section_index.contains_key(&key.to_ascii_lowercase())
    }

    pub fn section<'a>(&'a self, key: &str) -> Option<VSection<'a>> {
        self.section_index.get(&key.to_ascii_lowercase())
            .map(|indices| self.v_section(indices))
    }

    pub fn section_mut<'a>(&'a mut self, key: &str) -> Option<VSectionMut<'a>> {
        self.section_index.get(&key.to_ascii_lowercase())
            .cloned()
            .map(|indices| self.v_section_mut(&indices))
    }

    pub fn append_section<'a>(&'a mut self, key: &str) -> VSectionMut<'a> {
        let lower_key = key.to_ascii_lowercase();

        // Return section if it exists
        if self.section_index.contains_key(&lower_key) {
            return self.section_mut(&lower_key).unwrap();
        }

        // Create new section
        {
            let header = Item::Section(key.into(), ("", "\n").into());
            let section = Section::new(Rc::clone(&self.source), header);
            self.sections.push(section);
        }

        // Update index
        let indices = vec![self.sections.len() - 1];
        self.section_index.insert(lower_key, indices.clone());

        self.v_section_mut(&indices)
    }

    pub fn remove_section(&mut self, key: &str) {
        if self.has_section(key) {
            self.sections = self.sections.iter()
                .filter(|section| !section.key().eq_ignore_ascii_case(key))
                .cloned()
                .collect();
            self.section_index = Self::build_section_index(&self.sections);
        }
    }

    pub fn rename_section(&mut self, from_key: &str, to_key: &str) {
        let lower_to_key = to_key.to_ascii_lowercase();
        self.remove_section(&lower_to_key);

        let lower_from_key = from_key.to_ascii_lowercase();
        if let Some(indices) = self.section_index.remove(&lower_from_key) {
            let mut v_section = self.v_section_mut(&indices);
            v_section.set_key(to_key);
            self.section_index.insert(lower_to_key, indices);
        }
    }

    pub fn has_in(&self, section_key: &str, prop_key: &str) -> bool {
        self.section(section_key)
            .map_or(false, |section| section.has(prop_key))
    }

    pub fn get_in(&self, section_key: &str, prop_key: &str) -> Option<&str> {
        let lower_section_key = section_key.to_ascii_lowercase();
        self.section_index.get(&lower_section_key)
            .and_then(|indices| {
                indices.iter().rev()
                    .find_map(|&i| self.sections[i].get(prop_key))
            })
    }

    pub fn set_in(&mut self, section_key: &str, prop_key: &str, value: String) {
        let mut section = self.append_section(section_key);
        section.set(prop_key, value);
    }

    pub fn remove_in(&mut self, section_key: &str, prop_key: &str) {
        if let Some(mut section) = self.section_mut(section_key) {
            section.remove(prop_key);
        }
    }

    pub fn rename_in(&mut self, section_key: &str, from_key: &str, to_key: &str) {
        if let Some(mut section) = self.section_mut(section_key) {
            section.rename(from_key, to_key);
        }
    }
}

impl std::fmt::Display for Ini {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.global_section.to_string())?;
        for section in &self.sections {
            f.write_str(&section.to_string())?;
        }
        Ok(())
    }
}

fn borrow_indices<'a, T>(from: &'a [T], indices: &[usize]) -> Vec<&'a T> {
    indices.iter()
        .map(|&i| &from[i])
        .collect()
}

fn borrow_indices_mut<'a, T>(mut from: &'a mut [T], indices: &[usize]) -> Vec<&'a mut T> {
    let mut refs = Vec::with_capacity(indices.len());
    let mut rest = from;
    let mut rest_starts_at = 0;

    for i in indices {
        let borrow_at = i - rest_starts_at;
        let split_at = borrow_at + 1;
        (from, rest) = rest.split_at_mut(split_at);
        rest_starts_at += split_at;
        refs.push(&mut from[borrow_at]);
    }

    refs
}
