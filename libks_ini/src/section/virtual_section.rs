use super::{Section, SectionGroupIter};

#[derive(Debug)]
pub struct VirtualSection<'a> {
    sections: Vec<&'a Section>,
}

#[derive(Debug)]
pub struct VirtualSectionMut<'a> {
    sections: Vec<&'a mut Section>,
}

impl<'a> VirtualSection<'a> {
    pub(crate) fn new(sections: Vec<&'a Section>) -> Self {
        Self { sections }
    }

    pub fn key(&self) -> &str {
        self.sections[0].key()
    }

    pub fn has(&self, key: &str) -> bool {
        self.sections.iter().rev()
            .any(|section| section.has(key))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.sections.iter().rev()
            .find_map(|section| section.get(key))
    }

    pub fn iter(&'a self) -> SectionGroupIter<'a> {
        SectionGroupIter::new(self.sections.clone())
    }
}

impl<'a> VirtualSectionMut<'a> {
    pub(crate) fn new(sections: Vec<&'a mut Section>) -> Self {
        Self { sections }
    }

    pub fn key(&self) -> &str {
        self.sections[0].key()
    }

    pub fn set_key(&mut self, to_key: &str) {
        for section in &mut self.sections {
            section.set_key(to_key);
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.sections.iter().rev()
            .any(|section| section.has(key))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.sections.iter().rev()
            .find_map(|section| section.get(key))
    }

    pub fn set(&mut self, key: &str, mut value: String) {
        for section in self.sections.iter_mut().skip(1).rev() {
            match section.replace(key, value) {
                Some(value_temp) => value = value_temp,
                None => return,
            }
        }
        self.sections[0].set(key, value);
    }

    pub fn remove(&mut self, key: &str) {
        for section in &mut self.sections {
            section.remove(key);
        }
    }

    pub fn rename(&mut self, from_key: &str, to_key: &str) {
        for section in &mut self.sections {
            section.rename(from_key, to_key);
        }
    }
}
