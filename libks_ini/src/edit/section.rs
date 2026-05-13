use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use crate::{
    item::{Item, PropertyItem, SectionItem},
    span::Span,
    whitespace::{LineEnding, Padding2, Padding4},
};

#[derive(Debug, Clone)]
pub struct Section {
    pub(crate) header: SectionItem,
    pub(crate) items: Vec<Item>,
    pub(crate) line_ending: LineEnding,
}

#[derive(Debug, Clone)]
pub struct SectionReader<'a> {
    pub(crate) section: &'a Section,
    pub(crate) source: &'a str,
}

#[derive(Debug)]
pub struct SectionWriter<'a> {
    pub(crate) section: &'a mut Section,
    pub(crate) source: &'a str,
}

impl Section {
    pub fn new<S: Into<String>>(name: S, line_ending: LineEnding) -> Self {
        Self {
            header: SectionItem {
                key: Span::String(name.into()),
                padding: Padding2::default(),
                line_ending,
            },
            items: Vec::new(),
            line_ending,
        }
    }
    
    pub fn from_header(header: SectionItem) -> Self {
        let line_ending = header.line_ending;
        Self {
            header,
            items: Vec::new(),
            line_ending,
        }
    }
    
    pub fn header(&self) -> &SectionItem {
        &self.header
    }
    
    pub fn header_set_line_ending(&mut self, line_ending: LineEnding) {
        self.header.line_ending = line_ending;
    }
    
    pub fn header_set_padding<S1, S2>(&mut self, before: S1, after: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.header.padding.0 = Span::String(before.into());
        self.header.padding.1 = Span::String(after.into());
    }
    
    pub fn items_len(&self) -> usize {
        self.items.len()
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
    
    pub fn clear_items(&mut self) {
        self.items.clear()
    }
    
    pub fn iter_items(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }
    
    pub fn iter_items_mut(&mut self) -> std::slice::IterMut<'_, Item> {
        self.items.iter_mut()
    }
    
    pub fn into_items(self) -> <Vec<Item> as IntoIterator>::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for Section {
    type Item = Item;
    type IntoIter = <Vec<Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_items()
    }
}

impl<'a> IntoIterator for &'a Section {
    type Item = &'a Item;
    type IntoIter = std::slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_items()
    }
}

impl<'a> IntoIterator for &'a mut Section {
    type Item = &'a mut Item;
    type IntoIter = std::slice::IterMut<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_items_mut()
    }
}

impl<'a> SectionReader<'a> {
    pub fn new<S: AsRef<str> + ?Sized>(section: &'a Section, source: &'a S) -> Self {
        Self {
            section,
            source: source.as_ref(),
        }
    }
    
    pub fn key(&self) -> &'a str {
        self.section.header.key.to_str(self.source)
    }
    
    fn find_prop<K: AsRef<str>>(&self, key: K) -> Option<&'a PropertyItem> {
        for item in self.section.items.iter().rev() {
            let Item::Property(prop) = item else { continue };
            if prop.key.to_str(self.source)
                .eq_ignore_ascii_case(key.as_ref())
            {
                return Some(&prop);
            }
        }
        None
    }
    
    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.find_prop(key).is_some()
    }
    
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&'a str> {
        self.find_prop(key)
            .map(|prop| prop.value.to_str(self.source))
    }
    
    pub fn iter_props(&self) -> SectionPropsIter<'a> {
        SectionPropsIter::from(self)
    }
}

impl<'a> Deref for SectionReader<'a> {
    type Target = Section;

    fn deref(&self) -> &Self::Target {
        self.section
    }
}

impl<'a> IntoIterator for &'a SectionReader<'a> {
    type Item = (&'a str, &'a str);
    type IntoIter = SectionPropsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_props()
    }
}

impl<'a> SectionWriter<'a> {
    pub fn new<S: AsRef<str> + ?Sized>(section: &'a mut Section, source: &'a S) -> Self {
        Self {
            section,
            source: source.as_ref(),
        }
    }
    
    pub fn key(&self) -> &str {
        self.section.header.key.to_str(self.source)
    }
    
    fn find_prop<K: AsRef<str>>(&self, key: K) -> Option<&PropertyItem> {
        for item in self.items.iter().rev() {
            let Item::Property(prop) = item else { continue };
            if prop.key.to_str(self.source)
                .eq_ignore_ascii_case(key.as_ref())
            {
                return Some(&prop);
            }
        }
        None
    }
    
    fn find_prop_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut PropertyItem> {
        for item in self.section.items.iter_mut().rev() {
            let Item::Property(prop) = item else { continue };
            if prop.key.to_str(self.source)
                .eq_ignore_ascii_case(key.as_ref())
            {
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
            .map(|prop| prop.value.to_str(self.source))
    }
    
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str> + Into<String>,
        V: Into<String>,
    {
        match self.find_prop_mut(key.as_ref()) {
            Some(prop) => prop.value = Span::String(value.into()),
            None => {
                let index = self.find_index_for_append();
                let item = PropertyItem {
                    key: Span::String(key.into()),
                    value: Span::String(value.into()),
                    padding: Padding4::default(),
                    line_ending: self.line_ending,
                };
                self.items.insert(index, item.into());
            }
        }
    }
    
    pub fn replace<K, V>(&mut self, key: K, value: V) -> bool
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        match self.find_prop_mut(key.as_ref()) {
            Some(prop) => {
                prop.value = Span::String(value.into());
                true
            }
            None => false
        }
    }
    
    pub fn remove<K: AsRef<str>>(&mut self, key: K) {
        for i in (0..self.items.len()).rev() {
            let Item::Property(prop) = &self.items[i] else { continue };
            if prop.key.to_str(self.source)
                .eq_ignore_ascii_case(key.as_ref())
            {
                self.items.remove(i);
            }
        }
    }
    
    pub fn iter_props(&'a self) -> SectionPropsIter<'a> {
        SectionPropsIter::from(self)
    }
}

impl<'a> Deref for SectionWriter<'a> {
    type Target = Section;

    fn deref(&self) -> &Self::Target {
        self.section
    }
}

impl<'a> DerefMut for SectionWriter<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.section
    }
}

impl<'a> IntoIterator for &'a SectionWriter<'a> {
    type Item = (&'a str, &'a str);
    type IntoIter = SectionPropsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_props()
    }
}

pub struct SectionPropsIter<'a> {
    items: std::iter::Rev<std::slice::Iter<'a, Item>>,
    source: &'a str,
    keys_seen: HashSet<String>,
}

impl<'a> From<&SectionReader<'a>> for SectionPropsIter<'a> {
    fn from(reader: &SectionReader<'a>) -> Self {
        Self {
            items: reader.section.items.iter().rev(),
            source: reader.source,
            keys_seen: HashSet::new(),
        }
    }
}

impl<'a> From<&'a SectionWriter<'a>> for SectionPropsIter<'a> {
    fn from(writer: &'a SectionWriter<'a>) -> Self {
        Self {
            items: writer.section.items.iter().rev(),
            source: writer.source,
            keys_seen: HashSet::new(),
        }
    }
}

impl<'a> Iterator for SectionPropsIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.items.by_ref() {
            let Item::Property(prop) = item else { continue };
            let key = prop.key.to_str(self.source);
            let value = prop.value.to_str(self.source);
            if self.keys_seen.contains(key) {
                continue;
            }
            self.keys_seen.insert(key.to_owned());
            return Some((key, value));
        }
        None
    }
}
