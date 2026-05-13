use std::collections::HashSet;

use crate::item::Item;

use super::section::{SectionReader, SectionWriter};

#[derive(Debug)]
pub struct VirtualSection<'a> {
    sections: Vec<SectionReader<'a>>,
}

#[derive(Debug)]
pub struct VirtualSectionMut<'a> {
    sections: Vec<SectionWriter<'a>>,
}

impl<'a> VirtualSection<'a> {
    pub fn new(sections: Vec<SectionReader<'a>>) -> Self {
        Self {
            sections,
        }
    }

    pub fn key(&self) -> &'a str {
        self.sections.last()
            .map(|section| section.key())
            .unwrap_or("")
    }

    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.sections.iter()
            .rev()
            .any(|section| section.has(key.as_ref()))
    }

    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&'a str> {
        self.sections.iter()
            .rev()
            .find_map(|section| section.get(key.as_ref()))
    }

    pub fn iter_props(&'a self) -> VirtualSectionPropsIter<'a, SectionReader<'a>> {
        VirtualSectionPropsIter::from(self)
    }
}

impl<'a> VirtualSectionMut<'a> {
    pub fn new(sections: Vec<SectionWriter<'a>>) -> Self {
        Self {
            sections,
        }
    }

    pub fn key(&'a self) -> &'a str {
        self.sections.last()
            .map(|section| section.key())
            .unwrap_or("")
    }

    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.sections.iter()
            .rev()
            .any(|section| section.has(key.as_ref()))
    }

    pub fn get<K: AsRef<str>>(&'a self, key: K) -> Option<&'a str> {
        self.sections.iter()
            .rev()
            .find_map(|section| section.get(key.as_ref()))
    }

    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str> + Into<String>,
        V: Into<String>
    {
        let value = value.into();
        for section in self.sections.iter_mut().rev() {
            if section.replace(key.as_ref(), &value) {
                return;
            }
        }
        if let Some(section) = self.sections.last_mut() {
            section.set(key, value);
        }
    }

    pub fn unset<K: AsRef<str>>(&mut self, key: K) {
        for section in &mut self.sections {
            section.unset(key.as_ref());
        }
    }

    pub fn iter_props(&'a self) -> VirtualSectionPropsIter<'a, SectionWriter<'a>> {
        VirtualSectionPropsIter::from(self)
    }
}

#[allow(private_bounds)]
pub struct VirtualSectionPropsIter<'a, I>
where
    I: IterItems
{
    sections: std::iter::Rev<std::slice::Iter<'a, I>>,
    items: std::iter::Rev<std::slice::Iter<'a, Item>>,
    source: &'a str,
    keys_seen: HashSet<String>,
}

impl<'a> From<&'a VirtualSection<'a>> for VirtualSectionPropsIter<'a, SectionReader<'a>> {
    fn from(v_section: &'a VirtualSection<'a>) -> Self {
        let mut sections = v_section.sections.iter().rev();
        match sections.next() {
            Some(section) => Self {
                sections,
                items: section.iter_items().rev(),
                source: section.source,
                keys_seen: HashSet::new(),
            },
            None => Self {
                sections,
                items: [].iter().rev(),
                source: "",
                keys_seen: HashSet::new(),
            }
        }
    }
}

impl<'a> From<&'a VirtualSectionMut<'a>> for VirtualSectionPropsIter<'a, SectionWriter<'a>> {
    fn from(v_section: &'a VirtualSectionMut<'a>) -> Self {
        let mut sections = v_section.sections.iter().rev();
        match sections.next() {
            Some(section) => Self {
                sections,
                items: section.iter_items().rev(),
                source: section.source,
                keys_seen: HashSet::new(),
            },
            None => Self {
                sections,
                items: [].iter().rev(),
                source: "",
                keys_seen: HashSet::new(),
            }
        }
    }
}

impl<'a, I> Iterator for VirtualSectionPropsIter<'a, I>
where
    I: IterItems
{
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.items.by_ref() {
            let Item::Property(prop) = item else { continue };
            let key = prop.key.to_str(self.source);
            let value = prop.value.to_str(self.source);
            let key_lower = key.to_ascii_lowercase();
            if !self.keys_seen.insert(key_lower) {
                continue;
            }
            return Some((key, value));
        }
        
        let next_section = self.sections.next()?;
        self.items = next_section.iter_items().rev();
        self.next()
    }
}

trait IterItems {
    fn iter_items(&self) -> std::slice::Iter<'_, Item>;
}

impl<'a> IterItems for SectionReader<'a> {
    fn iter_items(&self) -> std::slice::Iter<'_, Item> {
        self.section.iter_items()
    }
}

impl<'a> IterItems for SectionWriter<'a> {
    fn iter_items(&self) -> std::slice::Iter<'_, Item> {
        self.section.iter_items()
    }
}
