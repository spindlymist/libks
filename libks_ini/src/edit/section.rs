use std::collections::HashSet;

use crate::{item::owned::{Item, PropertyItem, SectionItem}, whitespace::{LineEnding, owned::{Padding2, Padding4}}};

#[derive(Debug, Clone)]
pub struct Section {
    header: SectionItem,
    items: Vec<Item>,
    line_ending: LineEnding,
}

impl Section {
    pub fn new<S: Into<String>>(name: S, line_ending: LineEnding) -> Self {
        Self {
            header: SectionItem {
                key: name.into(),
                padding: Padding2::default(),
                line_ending,
            },
            items: Vec::new(),
            line_ending,
        }
    }
    
    pub fn from_header(header: SectionItem) -> Self {
        Self {
            line_ending: header.line_ending,
            header,
            items: Vec::new(),
        }
    }
    
    pub fn header(&self) -> &SectionItem {
        &self.header
    }
    
    pub fn header_mut(&mut self) -> &mut SectionItem {
        &mut self.header
    }
    
    pub fn key(&self) -> &str {
        &self.header.key
    }
    
    pub fn key_mut(&mut self) -> &mut String {
        &mut self.header.key
    }
    
    pub fn set_key<K: AsRef<str>>(&mut self, key: K) {
        self.header.key.clear();
        self.header.key.push_str(key.as_ref());
    }
    
    pub fn replace_key<K: Into<String>>(&mut self, key: K) {
        self.header.key = key.into();
    }
    
    fn find_prop<K: AsRef<str>>(&self, key: K) -> Option<&PropertyItem> {
        for item in self.items.iter().rev() {
            let Item::Property(prop) = item else { continue };
            if prop.key.eq_ignore_ascii_case(key.as_ref()) {
                return Some(&prop);
            }
        }
        None
    }
    
    fn find_prop_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut PropertyItem> {
        for item in self.items.iter_mut().rev() {
            let Item::Property(prop) = item else { continue };
            if prop.key.eq_ignore_ascii_case(key.as_ref()) {
                return Some(prop);
            }
        }
        None
    }
    
    fn find_index_for_append(&self) -> usize {
        for i in (0..self.items.len()).rev() {
            match self.items[i] {
                Item::Blank(_) => continue,
                _ => return i + 1,
            }
        }
        0
    }
    
    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.find_prop(key).is_some()
    }
    
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&str> {
        self.find_prop(key)
            .map(|prop| prop.value.as_str())
    }
    
    pub fn get_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut String> {
        self.find_prop_mut(key)
            .map(|prop| &mut prop.value)
    }
    
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str> + Into<String>,
        V: Into<String>,
    {
        match self.find_prop_mut(key.as_ref()) {
            Some(prop) => prop.value = value.into(),
            None => {
                let index = self.find_index_for_append();
                let item = PropertyItem {
                    key: key.into(),
                    value: value.into(),
                    padding: Padding4::default(),
                    line_ending: self.line_ending,
                };
                self.items.insert(index, item.into());
            }
        }
    }
    
    pub fn remove<K: AsRef<str>>(&mut self, key: K) {
        for i in (0..self.items.len()).rev() {
            let Item::Property(prop) = &self.items[i] else { continue };
            if prop.key.eq_ignore_ascii_case(key.as_ref()) {
                self.items.remove(i);
            }
        }
    }
    
    pub fn get_item(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }
    
    pub fn get_item_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }
    
    pub fn append_item(&mut self, item: Item) {
        self.items.push(item);
    }
    
    pub fn insert_item(&mut self, index: usize, item: Item) {
        self.items.insert(index, item);
    }
    
    pub fn extend_items<T>(&mut self, items: T)
    where
        T: IntoIterator<Item = Item>
    {
        self.items.extend(items);
    }
    
    pub fn remove_item(&mut self, index: usize) -> Item {
        self.items.remove(index)
    }
    
    pub fn iter_props(&self) -> SectionPropsIter<'_> {
        SectionPropsIter::new(self)
    }
    
    pub fn iter_props_mut(&mut self) -> SectionPropsIterMut<'_> {
        SectionPropsIterMut::new(self)
    }
    
    pub fn iter_items(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }
    
    pub fn iter_items_mut(&mut self) -> std::slice::IterMut<'_, Item> {
        self.items.iter_mut()
    }
}

impl<'a> IntoIterator for &'a Section {
    type Item = (&'a str, &'a str);
    type IntoIter = SectionPropsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_props()
    }
}

impl<'a> IntoIterator for &'a mut Section {
    type Item = (&'a mut String, &'a mut String);
    type IntoIter = SectionPropsIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_props_mut()
    }
}

pub struct SectionPropsIter<'a> {
    inner: std::iter::Rev<std::slice::Iter<'a, Item>>,
    keys_seen: HashSet<String>,
}

impl<'a> SectionPropsIter<'a> {
    pub fn new(section: &'a Section) -> Self {
        Self {
            inner: section.iter_items().rev(),
            keys_seen: HashSet::new(),
        }
    }
}

impl<'a> Iterator for SectionPropsIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.inner.by_ref() {
            let Item::Property(prop) = item else { continue };
            let key = prop.key.as_str();
            let value = prop.value.as_str();
            if self.keys_seen.contains(key) {
                continue;
            }
            self.keys_seen.insert(prop.key.clone());
            return Some((key, value));
        }
        None
    }
}

pub struct SectionPropsIterMut<'a> {
    inner: std::iter::Rev<std::slice::IterMut<'a, Item>>,
    keys_seen: HashSet<String>,
}

impl<'a> SectionPropsIterMut<'a> {
    pub fn new(section: &'a mut Section) -> Self {
        Self {
            inner: section.iter_items_mut().rev(),
            keys_seen: HashSet::new(),
        }
    }
}

impl<'a> Iterator for SectionPropsIterMut<'a> {
    type Item = (&'a mut String, &'a mut String);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.inner.by_ref() {
            let Item::Property(prop) = item else { continue };
            if self.keys_seen.contains(&prop.key) {
                continue;
            }
            self.keys_seen.insert(prop.key.clone());
            return Some((&mut prop.key, &mut prop.value));
        }
        None
    }
}
