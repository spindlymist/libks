use std::{
    cell::{Ref, RefCell},
    collections::{hash_map::Entry::*, HashMap},
    rc::Rc,
};

use crate::{
    item::Item,
    section::{ConcreteSection, VirtualSection},
};

macro_rules! make_ptr {
    ( $inner:expr ) => {
        Rc::new(RefCell::new($inner))
    };
}

pub struct Ini {
    source: Rc<str>,
    global_section: ConcreteSection,
    sections: Vec<Rc<RefCell<ConcreteSection>>>,
    virtual_sections: HashMap<String, VirtualSection>,
}

impl Ini {
    pub fn new(source: &str) -> Self {
        let source = Rc::<str>::from(source);
        let mut sections = Vec::new();
        let mut virtual_sections = HashMap::<_, VirtualSection>::new();
        let mut global_section = ConcreteSection::new_global(Rc::clone(&source));
        let mut current_section: Option<Rc<RefCell<ConcreteSection>>> = None;

        for item in crate::parse::Parser::new(&source).map(Item::from) {
            match item {
                Item::Section(key, padding) => {
                    let lower_key = key.of(&source).to_ascii_lowercase();
                    let header = Item::Section(key, padding);
                    let section = make_ptr![ConcreteSection::new(Rc::clone(&source), header)];
                    sections.push(Rc::clone(&section));
                    
                    match virtual_sections.entry(lower_key) {
                        Occupied(mut entry) => {
                            entry.get_mut().push_section(Rc::clone(&section));
                        },
                        Vacant(entry) => {
                            let v_section = VirtualSection::new(Rc::clone(&section));
                            entry.insert(v_section);
                        }
                    }

                    current_section = Some(section);
                },
                _ => match &current_section {
                    Some(section) => section.borrow_mut().push_item(item),
                    None => global_section.push_item(item),
                },
            }
        }

        Self {
            source,
            global_section,
            sections,
            virtual_sections,
        }
    }

    pub fn has_section(&self, key: &str) -> bool {
        self.virtual_sections.contains_key(&key.to_ascii_lowercase())
    }

    pub fn get_section(&self, key: &str) -> Option<&VirtualSection> {
        self.virtual_sections.get(&key.to_ascii_lowercase())
    }

    pub fn get_section_mut(&mut self, key: &str) -> Option<&mut VirtualSection> {
        self.virtual_sections.get_mut(&key.to_ascii_lowercase())
    }

    pub fn append_section(&mut self, key: &str) -> &mut VirtualSection {
        match self.virtual_sections.entry(key.to_ascii_lowercase()) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => {
                let header = Item::Section(key.to_owned().into(), ("", "\n").into());
                let section = make_ptr![ConcreteSection::new(Rc::clone(&self.source), header)];
                let v_section = VirtualSection::new(Rc::clone(&section));
                self.sections.push(section);
                entry.insert(v_section)
            },
        }
    }

    pub fn remove_section(&mut self, key: &str) {
        if self.virtual_sections.remove(key).is_some() {
            self.sections = self.sections.iter()
                .filter(|section| section.borrow().key() != key)
                .cloned()
                .collect();
        }
    }

    pub fn rename_section(&mut self, from_key: &str, to_key: &str) {
        self.remove_section(to_key);
        let lower_key = from_key.to_ascii_lowercase();
        if let Some(mut v_section) = self.virtual_sections.remove(&lower_key) {
            v_section.set_key(to_key);
            self.virtual_sections.insert(lower_key, v_section);
        }
    }

    pub fn has_in_section(&self, section_key: &str, prop_key: &str) -> bool {
        self.get_section(section_key)
            .map_or(false, |section| section.has(prop_key))
    }

    pub fn get_from_section(&self, section_key: &str, prop_key: &str) -> Option<Ref<str>> {
        self.get_section(section_key)
            .and_then(|section| section.get(prop_key))
    }

    pub fn set_in_section(&mut self, section_key: &str, prop_key: &str, value: String) {
        let section = self.append_section(section_key);
        section.set(prop_key, value);
    }

    pub fn remove_from_section(&mut self, section_key: &str, prop_key: &str) {
        if let Some(section) = self.get_section_mut(section_key) {
            section.remove(prop_key);
        }
    }

    pub fn rename_in_section(&mut self, section_key: &str, from_key: &str, to_key: &str) {
        if let Some(section) = self.get_section_mut(section_key) {
            section.rename(from_key, to_key);
        }
    }
}

impl std::fmt::Display for Ini {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.global_section.to_string())?;
        for section in &self.sections {
            f.write_str(&section.borrow().to_string())?;
        }
        Ok(())
    }
}
