use std::{collections::HashMap, fmt};

use crate::{Parser, edit::{VirtualSection, VirtualSectionMut}, item::owned::{Item, SectionItem}, whitespace::{LineEnding, owned::Padding2}};
use super::section::Section;

#[derive(Debug, Clone)]
pub struct Ini {
    global_section: Section,
    sections: Vec<Section>,
    line_ending: LineEnding,
}

impl Ini {
    pub fn new(line_ending: LineEnding) -> Self {
        Ini {
            global_section: Section::new(String::new(), line_ending),
            sections: Vec::new(),
            line_ending,
        }
    }
    
    fn find_section<K: AsRef<str>>(&self, key: K) -> Option<&Section> {
        for section in self.sections.iter().rev() {
            if section.key().eq_ignore_ascii_case(key.as_ref()) {
                return Some(section);
            }
        }
        None
    }
    
    fn find_section_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut Section> {
        for section in self.sections.iter_mut().rev() {
            if section.key().eq_ignore_ascii_case(key.as_ref()) {
                return Some(section);
            }
        }
        None
    }
    
    pub fn has_section<K: AsRef<str>>(&self, key: K) -> bool {
        self.find_section(key).is_some()
    }
    
    pub fn v_section<K>(&self, key: K) -> Option<VirtualSection<'_>>
    where
        K: AsRef<str> + Into<String>
    {
        VirtualSection::new(key, &self.sections)
    }
    
    pub fn v_section_mut<K>(&mut self, key: K) -> Option<VirtualSectionMut<'_>>
    where
        K: AsRef<str> + Into<String>
    {
        VirtualSectionMut::new(key, &mut self.sections)
    }
    
    pub fn section_at(&self, index: usize) -> Option<&Section> {
        self.sections.get(index)
    }
    
    pub fn section_at_mut(&mut self, index: usize) -> Option<&mut Section> {
        self.sections.get_mut(index)
    }
    
    pub fn insert_section<K: Into<String>>(&mut self, index: usize, key: K) -> &mut Section {
        let section = Section::new(key.into(), self.line_ending);
        self.sections.insert(index, section);
        &mut self.sections[index]
    }
    
    pub fn append_section<K: Into<String>>(&mut self, key: K) -> &mut Section {
        let section = Section::new(key.into(), self.line_ending);
        self.sections.push(section);
        let index = self.sections.len() - 1;
        &mut self.sections[index]
    }
    
    pub fn remove_section_at(&mut self, index: usize) -> Section {
        self.sections.remove(index)
    }
    
    pub fn remove_sections<K: AsRef<str>>(&mut self, key: K) {
        for i in (0..self.sections.len()).rev() {
            if self.sections[i].key().eq_ignore_ascii_case(key.as_ref()) {
                self.sections.remove(i);
            }
        }
    }
    
    pub fn iter_sections(&self) -> std::slice::Iter<'_, Section> {
        self.sections.iter()
    }
    
    pub fn iter_sections_mut(&mut self) -> std::slice::IterMut<'_, Section> {
        self.sections.iter_mut()
    }
}

impl<'a> From<&'a str> for Ini {
    fn from(source: &'a str) -> Self {
        let mut sections = Vec::new();
        let mut global_section = Section::new(String::new(), LineEnding::default());
        let mut current_section = &mut global_section;
        let mut line_endings = [0, 0, 0, 0];
        
        for item in Parser::new(source) {
            let item = item.into_owned(source);
            match item {
                Item::Section(header) => {
                    line_endings[header.line_ending as usize] += 1;
                    
                    let section = Section::from_header(header);
                    sections.push(section);
                    
                    let index = sections.len() - 1;
                    current_section = &mut sections[index];
                },
                _ => {
                    current_section.append_item(item);
                }
            }
        }

        let mut line_ending = LineEnding::default();
        let mut max_count = 0;
        for (i, count) in line_endings.iter().enumerate().skip(1) {
            if *count > max_count {
                max_count = *count;
                line_ending = match i {
                    1 => LineEnding::Cr,
                    2 => LineEnding::Lf,
                    3 => LineEnding::CrLf,
                    _ => LineEnding::default(),
                };
            }
        }
        global_section.header_mut().line_ending = line_ending;
        
        Self {
            global_section,
            sections,
            line_ending,
        }
    }
}

impl<'a> From<String> for Ini {
    fn from(source: String) -> Self {
        Self::from(source.as_str())
    }
}

impl fmt::Display for Ini {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.global_section.iter_items() {
            item.fmt(f)?;
        }
        for section in self.iter_sections() {
            section.header().fmt(f)?;
            for item in section.iter_items() {
                item.fmt(f)?;
            }
        }
        Ok(())
    }
}
