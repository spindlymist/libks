use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use super::ConcreteSection;

#[derive(Debug, Default)]
pub struct VirtualSection {
    sections: Vec<Rc<RefCell<ConcreteSection>>>,
}

impl VirtualSection {
    pub fn new(first_section: Rc<RefCell<ConcreteSection>>) -> Self {
        Self {
            sections: vec![first_section],
        }
    }

    pub fn push_section(&mut self, section: Rc<RefCell<ConcreteSection>>) {
        self.sections.push(section);
    }

    pub fn key(&self) -> Ref<str> {
        Ref::map(self.sections[0].borrow(), |section| section.key())
    }

    pub fn set_key(&mut self, to_key: &str) {
        for section in &mut self.sections {
            section.borrow_mut().set_key(to_key);
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.sections.iter().rev()
            .any(|section| section.borrow().has(key))
    }

    pub fn get(&self, key: &str) -> Option<Ref<str>> {
        for section in self.sections.iter().rev() {
            if let Ok(value) = Ref::filter_map(section.borrow(), |section| section.get(key)) {
                return Some(value);
            }
        }
        None
    }

    pub fn set(&mut self, key: &str, mut value: String) {
        for section in self.sections.iter_mut().skip(1).rev() {
            match section.borrow_mut().replace(key, value) {
                Some(value2) => value = value2,
                None => return,
            }
        }

        if let Some(section) = self.sections.first() {
            section.borrow_mut().set(key, value);
        }
    }

    pub fn remove(&mut self, key: &str) {
        for section in &mut self.sections {
            section.borrow_mut().remove(key);
        }
    }

    pub fn rename(&mut self, from_key: &str, to_key: &str) {
        for section in &mut self.sections {
            section.borrow_mut().rename(from_key, to_key);
        }
    }
}
