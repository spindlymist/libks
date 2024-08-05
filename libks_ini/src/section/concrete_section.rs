use std::rc::Rc;

use crate::{item::{
    Item, ItemsIteratorExt, Padding4, Prop
}, span::Span};

#[derive(Debug, Clone)]
pub struct ConcreteSection {
    source: Rc<str>,
    items: Vec<Item>,
}

impl ConcreteSection {
    pub fn new(source: Rc<str>, header: Item) -> Self {
        let items = match header {
            Item::Section(..) => vec![header],
            _ => panic!("Section header item must be Section variant"),
        };

        Self {
            source,
            items,
        }
    }

    pub fn new_global(source: Rc<str>) -> Self {
        Self {
            source,
            items: Vec::new(),
        }
    }

    pub fn push_item(&mut self, item: Item) {
        self.items.push(item);
    }

    /// # Panics
    /// 
    /// This method panics if called on the global section.
    pub fn key(&self) -> &str {
        match &self.items[0] {
            Item::Section(key, _) => key.of(&self.source),
            _ => panic!("ConcreteSection::key cannot be called on the global section"),
        }
    }

    /// # Panics
    /// 
    /// This method panics if called on the global section.
    pub fn key_span(&self) -> &Span {
        match &self.items[0] {
            Item::Section(key, _) => key,
            _ => panic!("ConcreteSection::key_span cannot be called on the global section"),
        }
    }

    /// # Panics
    /// 
    /// This method panics if called on the global section.
    pub fn set_key(&mut self, to_key: &str) {
        match &mut self.items[0] {
            Item::Section(key, _) => *key = to_key.into(),
            _ => panic!("ConcreteSection::set_key cannot be called on the global section"),
        }
    }

    fn find_prop(&self, key: &str) -> Option<&Prop> {
        for item in self.items.iter().rev() {
            if let Item::Property(prop, _) = item {
                if prop.key.of(&self.source).eq_ignore_ascii_case(key) {
                    return Some(prop);
                }
            }
        }
        None
    }

    fn find_prop_mut(&mut self, key: &str) -> Option<&mut Prop> {
        for item in self.items.iter_mut().rev() {
            if let Item::Property(prop, _) = item {
                if prop.key.of(&self.source).eq_ignore_ascii_case(key) {
                    return Some(prop);
                }
            }
        }
        None
    }

    pub fn has(&self, key: &str) -> bool {
        self.find_prop(key).is_some()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.find_prop(key)
            .map(|prop| prop.value.of(&self.source))
    }

    pub fn get_span(&self, key: &str) -> Option<&Span> {
        self.find_prop(key)
            .map(|prop| &prop.value)
    }

    pub fn set(&mut self, key: &str, value: String) {
        if let Some(kvp) = self.find_prop_mut(key) {
            kvp.value = value.into();
        }
        else {
            let item = Item::Property(
                Prop::from((key, value)),
                Padding4::from(("", "", "", "\n")),
            );
            self.items.push(item);
        }
    }

    pub fn replace(&mut self, key: &str, value: String) -> Option<String> {
        if let Some(kvp) = self.find_prop_mut(key) {
            kvp.value = value.into();
            None
        }
        else {
            Some(value)
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.items = self.items.iter()
            .filter(|item| match item {
                Item::Property(prop, _) => prop.key.of(&self.source).eq_ignore_ascii_case(key),
                _ => true,
            })
            .cloned()
            .collect();
    }

    pub fn rename(&mut self, from_key: &str, to_key: &str) {
        self.remove(to_key);
        for item in &mut self.items {
            match item {
                Item::Property(prop, _) if prop.key.of(&self.source).eq_ignore_ascii_case(from_key) => {
                    prop.key = to_key.into();
                }
                _ => (),
            }
        }
    }
}

impl std::fmt::Display for ConcreteSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = self.items.iter()
            .with_source(&self.source)
            .collect::<String>();
        f.write_str(&output)
    }
}
