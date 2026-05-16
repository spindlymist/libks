use std::fmt;

use crate::{
    Parser,
    edit::{
        SectionReader,
        SectionWriter,
        section_map::SectionMap,
        logical_section::{LogicalSection, LogicalSectionMut},
    },
    item::Item,
    span::Span,
    whitespace::LineEnding,
};
use super::section::Section;

#[derive(Debug, Clone)]
pub struct Ini {
    pub(crate) source: String,
    pub(crate) global_section: Section,
    pub(crate) sections: Vec<Section>,
    pub(crate) section_map: SectionMap,
    pub(crate) line_ending: LineEnding,
}

impl Ini {
    pub fn new(line_ending: LineEnding) -> Self {
        Ini {
            source: String::new(),
            global_section: Section::new("", line_ending),
            sections: Vec::new(),
            section_map: SectionMap::new(),
            line_ending,
        }
    }
    
    pub fn parse<S: Into<String>>(source: S) -> Self {
        let source = source.into();
        let mut parser = Parser::new(&source).peekable();
        
        let first_line_ending = match parser.peek() {
            Some(Item::Section(inner)) => inner.line_ending,
            Some(Item::Property(inner)) => inner.line_ending,
            Some(Item::Comment(inner)) => inner.line_ending,
            Some(Item::Blank(inner)) => inner.line_ending,
            Some(Item::Error(inner)) => inner.line_ending,
            None => LineEnding::default(),
        };
        
        let mut sections = Vec::new();
        let mut global_section = Section::new(String::new(), first_line_ending);
        let mut current_section = &mut global_section;
        
        for item in parser {
            match item {
                Item::Section(header) => {
                    let section = Section::from_header(header);
                    sections.push(section);
                    let index = sections.len() - 1;
                    current_section = &mut sections[index];
                }
                _ => current_section.append_item(item)
            }
        }
        
        let mut section_map = SectionMap::new();
        section_map.rebuild(&sections, &source);
        
        Self {
            source,
            global_section,
            sections,
            section_map,
            line_ending: first_line_ending,
        }
    }
    
    pub fn enable_indexing(&mut self) {
        self.section_map.is_enabled = true;
        if self.section_map.is_dirty {
            self.section_map.rebuild(&self.sections, &self.source);
        }
    }
    
    pub fn disable_indexing(&mut self) {
        self.section_map.is_enabled = false;
    }
    
    pub fn source(&self) -> &str {
        &self.source
    }
    
    pub fn len(&self) -> usize {
        self.sections.len()
    }
    
    pub fn global_section(&self) -> SectionReader<'_> {
        SectionReader {
            section: &self.global_section,
            source: &self.source,
        }
    }
    
    pub fn global_section_mut(&mut self) -> SectionWriter<'_> {
        SectionWriter {
            section: &mut self.global_section,
            source: &self.source,
        }
    }
    
    pub fn section_indices<K: AsRef<str>>(&self, key: K) -> Vec<usize> {
        if self.section_map.is_dirty {
            self.sections.iter()
                .enumerate()
                .filter(|(_i, section)| {
                    section.header.key.to_str(&self.source)
                        .eq_ignore_ascii_case(key.as_ref()) 
                })
                .map(|(i, _section)| i)
                .collect()
        }
        else if let Some(indices) = self.section_map.get(key.as_ref()) {
            Vec::from(indices)
        }
        else {
            Vec::new()
        }
    }
    
    fn find_sections<K: AsRef<str>>(&self, key: K) -> Vec<SectionReader<'_>> {
        if self.section_map.is_dirty {
            self.sections.iter()
                .filter(|section| {
                    section.header.key.to_str(&self.source)
                        .eq_ignore_ascii_case(key.as_ref()) 
                })
                .map(|section| SectionReader::new(section, &self.source))
                .collect()
        }
        else if let Some(indices) = self.section_map.get(key.as_ref()) {
            indices.iter()
                .map(|i| SectionReader::new(&self.sections[*i], &self.source))
                .collect()
        }
        else {
            Vec::new()
        }
    }
    
    fn find_sections_mut<K: AsRef<str>>(&mut self, key: K) -> Vec<SectionWriter<'_>> {
        if self.section_map.is_dirty {
            self.sections.iter_mut()
                .filter(|section| {
                    section.header.key.to_str(&self.source)
                        .eq_ignore_ascii_case(key.as_ref()) 
                })
                .map(|section| SectionWriter::new(section, &self.source))
                .collect()
        }
        else if let Some(indices) = self.section_map.get(key.as_ref()) {
            let mut sections = Vec::with_capacity(indices.len());
            let mut left;
            let mut right = self.sections.as_mut_slice();
            let mut right_start_index = 0;

            for i in indices {
                let borrow_at = i - right_start_index;
                let split_at = borrow_at + 1;
                (left, right) = right.split_at_mut(split_at);
                right_start_index += split_at;
                sections.push(SectionWriter::new(&mut left[borrow_at], &self.source));
            }

            sections
        }
        else {
            Vec::new()
        }
    }
    
    pub fn has_section<K: AsRef<str>>(&self, key: K) -> bool {
        if self.section_map.is_dirty {
            self.sections.iter()
                .any(|section| {
                    section.header.key.to_str(&self.source)
                        .eq_ignore_ascii_case(key.as_ref())     
                })
        }
        else {
            self.section_map.has(key)
        }
    }
    
    pub fn section_at(&self, index: usize) -> Option<SectionReader<'_>> {
        let section = self.sections.get(index)?;
        Some(SectionReader::new(section, &self.source))
    }
    
    pub fn section_at_mut(&mut self, index: usize) -> Option<SectionWriter<'_>> {
        let section = self.sections.get_mut(index)?;
        Some(SectionWriter::new(section, &self.source))
    }
    
    pub fn section<K>(&self, key: K) -> Option<LogicalSection<'_>>
    where
        K: AsRef<str> + Into<String>
    {
        let sections = self.find_sections(key);
        if sections.is_empty() {
            None
        }
        else {
            Some(LogicalSection::new(sections))
        }
    }
    
    pub fn section_mut<K>(&mut self, key: K) -> Option<LogicalSectionMut<'_>>
    where
        K: AsRef<str> + Into<String>
    {
        let sections = self.find_sections_mut(key);
        if sections.is_empty() {
            return None;
        }
        Some(LogicalSectionMut::new(sections))
    }
    
    pub fn insert_section<K: Into<String>>(&mut self, index: usize, key: K) -> SectionWriter<'_> {
        let key = key.into();
        self.section_map.update_after_insert(&key, index);
        
        let section = Section::new(key, self.line_ending);
        self.sections.insert(index, section);
        
        SectionWriter::new(&mut self.sections[index], &self.source)
    }
    
    pub fn append_section<K: Into<String>>(&mut self, key: K) -> SectionWriter<'_> {
        let key = key.into();
        let index = self.sections.len();
        self.section_map.update_after_append(&key, index);
        
        let section = Section::new(key, self.line_ending);
        self.sections.push(section);
        
        SectionWriter::new(&mut self.sections[index], &self.source)
    }
    
    pub fn remove_section_at(&mut self, index: usize) -> Section {
        let section = self.sections.remove(index);
        let key = section.header.key.to_str(&self.source);
        self.section_map.update_after_remove(key, index);
        
        section
    }
    
    pub fn remove_sections<K: AsRef<str>>(&mut self, key: K) -> Vec<usize> {
        let indices = self.section_indices(key);
        for &i in indices.iter().rev() {
            self.remove_section_at(i);
        }
        
        indices
    }
    
    pub fn rename_section_at<K: Into<String>>(&mut self, index: usize, key: K) {
        let section = &mut self.sections[index];
        let old_key = section.header.key.to_str(&self.source).to_owned();
        let new_key = key.into();
        self.section_map.update_after_rename(index, old_key, &new_key);
        
        section.header.key = Span::String(new_key);
    }
    
    pub fn rename_sections<K1, K2>(&mut self, key_from: K1, key_to: K2) -> Vec<usize>
    where
        K1: AsRef<str>,
        K2: Into<String>,
    {
        let indices = self.section_indices(key_from);
        let key_to = key_to.into();
        
        if indices.len() == 1 {
            self.rename_section_at(indices[0], key_to)
        }
        else {
            for &i in &indices {
                self.rename_section_at(i, key_to.clone());
            }
        }
        
        indices
    }
    
    pub fn clear(&mut self) {
        self.global_section.clear_items();
        self.sections.clear();
        self.section_map.clear();
    }
    
    pub fn iter_sections(&self) -> impl Iterator<Item = SectionReader<'_>> {
        self.sections.iter()
            .map(|section| SectionReader::new(section, &self.source))
    }
    
    pub fn iter_sections_mut(&mut self) -> impl Iterator<Item = SectionWriter<'_>> {
        self.sections.iter_mut()
            .map(|section| SectionWriter::new(section, &self.source))
    }
}

impl<'a> From<&'a str> for Ini {
    fn from(source: &'a str) -> Self {
        Ini::parse(source.to_owned())
    }
}

impl From<String> for Ini {
    fn from(source: String) -> Self {
        Ini::parse(source)
    }
}

impl fmt::Display for Ini {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.global_section.iter_items() {
            item.with_source(&self.source).fmt(f)?;
        }
        for section in self.iter_sections() {
            section.fmt(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_macros::*;
    use crate::whitespace::LineEnding;
    use crate::item::{Item, ItemsIterExt};
    
    #[test]
    fn ini_parse_works() {
        const SOURCE: &'static str = "\
; This is the global section
Global Prop 0=Global Section/Global Prop 0
Global Prop 1=Global Section/Global Prop 1

an error? in my global section?

[Section 0]
; This is section 0
Prop 0=Section 0/Prop 0
Prop 1=Section 0/Prop 1

[Section 1]
; This is section 1
Prop 3=Section 1/Prop 3
Prop 4=Section 1/Prop 4
";
        let ini = Ini::parse(SOURCE);
        
        assert_eq!(ini.len(), 2);
        {
            let section = ini.global_section();
            let expected = [
                comment!(" This is the global section"),
                prop!("Global Prop 0" => "Global Section/Global Prop 0"),
                prop!("Global Prop 1" => "Global Section/Global Prop 1"),
                blank!(),
                error!("an error? in my global section?"),
                blank!(),
            ];
            assert_eq_iter!(
                section.iter_items().with_source(ini.source()),
                expected.iter().with_source(ini.source())
            );
        }
        {
            let section = ini.section_at(0).unwrap();
            assert_eq!(
                Item::from(section.header().clone()).with_source(ini.source()),
                section!("Section 0").with_source(ini.source())
            );
            let expected = [
                comment!(" This is section 0"),
                prop!("Prop 0" => "Section 0/Prop 0"),
                prop!("Prop 1" => "Section 0/Prop 1"),
                blank!(),
            ];
            assert_eq_iter!(
                section.iter_items().with_source(ini.source()),
                expected.iter().with_source(ini.source())
            );
        }
        {
            let section = ini.section_at(1).unwrap();
            assert_eq!(
                Item::from(section.header().clone()).with_source(ini.source()),
                section!("Section 1").with_source(ini.source())
            );
            let expected = [
                comment!(" This is section 1"),
                prop!("Prop 3" => "Section 1/Prop 3"),
                prop!("Prop 4" => "Section 1/Prop 4"),
                blank!("", end=LineEnding::None),
            ];
            assert_eq_iter!(
                section.iter_items().with_source(ini.source()),
                expected.iter().with_source(ini.source())
            );
        }
    }
    
    #[test]
    fn ini_display_works() {
        const SOURCE: &'static str = before!("the_machine.ini");
        let ini = Ini::parse(SOURCE);
        assert_eq!(ini.to_string(), SOURCE);
    }
    
    #[test]
    fn ini_section_indices_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        
        assert_eq!(ini.section_indices("Section 0"), [0, 3, 6]);
        assert_eq!(ini.section_indices("Section 1"), [1, 4, 7]);
        assert_eq!(ini.section_indices("Section 2"), [2, 5, 8]);
        assert_eq!(ini.section_indices("Section 3"), []);
    }
    
    #[test]
    fn ini_has_section_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        
        assert!(ini.has_section("Section 0"));
        assert!(ini.has_section("Section 1"));
        assert!(ini.has_section("Section 2"));
        assert!(!ini.has_section("Section 3"));
    }
    
    #[test]
    fn ini_has_section_works_with_dirty_map() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        ini.section_map.is_dirty = true;
        
        assert!(ini.has_section("Section 0"));
        assert!(ini.has_section("Section 1"));
        assert!(ini.has_section("Section 2"));
        assert!(!ini.has_section("Section 3"));
    }
    
    #[test]
    fn ini_find_sections_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        
        let sections = ini.find_sections("Section 0");
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].key(), "Section 0");
        assert_eq!(sections[1].key(), "SECTION 0");
        assert_eq!(sections[2].key(), "section 0");
    }
    
    #[test]
    fn ini_find_sections_works_with_dirty_map() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        ini.section_map.is_dirty = true;
        
        let sections = ini.find_sections("Section 0");
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].key(), "Section 0");
        assert_eq!(sections[1].key(), "SECTION 0");
        assert_eq!(sections[2].key(), "section 0");
    }
    
    #[test]
    fn ini_find_sections_mut_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        
        let sections = ini.find_sections_mut("Section 0");
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].key(), "Section 0");
        assert_eq!(sections[1].key(), "SECTION 0");
        assert_eq!(sections[2].key(), "section 0");
    }
    
    #[test]
    fn ini_find_sections_mut_works_with_dirty_map() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        ini.section_map.is_dirty = true;
        
        let sections = ini.find_sections_mut("Section 0");
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].key(), "Section 0");
        assert_eq!(sections[1].key(), "SECTION 0");
        assert_eq!(sections[2].key(), "section 0");
    }
    
    #[test]
    fn ini_section_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        
        let logical_section = ini.section("Section 0").unwrap();
        assert_eq!(logical_section.sections.len(), 3);
        assert_eq!(logical_section.sections[0].key(), "Section 0");
        assert_eq!(logical_section.sections[1].key(), "SECTION 0");
        assert_eq!(logical_section.sections[2].key(), "section 0");

        assert!(ini.section("Section 3").is_none());
    }
    
    #[test]
    fn ini_section_works_with_dirty_map() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        ini.section_map.is_dirty = true;
        
        let logical_section = ini.section("Section 0").unwrap();
        assert_eq!(logical_section.sections.len(), 3);
        assert_eq!(logical_section.sections[0].key(), "Section 0");
        assert_eq!(logical_section.sections[1].key(), "SECTION 0");
        assert_eq!(logical_section.sections[2].key(), "section 0");

        assert!(ini.section("Section 3").is_none());
    }
    
    #[test]
    fn ini_section_mut_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        
        let logical_section = ini.section_mut("Section 0").unwrap();
        assert_eq!(logical_section.sections.len(), 3);
        assert_eq!(logical_section.sections[0].key(), "Section 0");
        assert_eq!(logical_section.sections[1].key(), "SECTION 0");
        assert_eq!(logical_section.sections[2].key(), "section 0");

        assert!(ini.section_mut("Section 3").is_none());
    }
    
    #[test]
    fn ini_section_mut_works_with_dirty_map() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        ini.section_map.is_dirty = true;
        
        let logical_section = ini.section_mut("Section 0").unwrap();
        assert_eq!(logical_section.sections.len(), 3);
        assert_eq!(logical_section.sections[0].key(), "Section 0");
        assert_eq!(logical_section.sections[1].key(), "SECTION 0");
        assert_eq!(logical_section.sections[2].key(), "section 0");

        assert!(ini.section_mut("Section 3").is_none());
    }
    
    #[test]
    fn ini_insert_section_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        {
            let mut ini = ini.clone();
            ini.insert_section(0, "secTION 0");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "secTION 0",
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "SECTION 2",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        {
            let mut ini = ini.clone();
            ini.insert_section(5, "Section 3");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "Section 3",
                "SECTION 2",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
    }
    
    #[test]
    fn ini_append_section_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        {
            let mut ini = ini.clone();
            ini.append_section("secTION 0");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "SECTION 2",
                "section 0",
                "section 1",
                "section 2",
                "secTION 0",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        {
            let mut ini = ini.clone();
            ini.append_section("Section 3");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "SECTION 2",
                "section 0",
                "section 1",
                "section 2",
                "Section 3",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
    }
    
    #[test]
    fn ini_remove_section_at_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        {
            let mut ini = ini.clone();
            ini.remove_section_at(0);
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "SECTION 2",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        {
            let mut ini = ini.clone();
            ini.remove_section_at(5);
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
    }
    
    #[test]
    fn ini_remove_sections_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        {
            let mut ini = ini.clone();
            let indices = ini.remove_sections("secTION 0");
            assert_eq!(indices, [0, 3, 6]);
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 1",
                "Section 2",
                "SECTION 1",
                "SECTION 2",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        // Removing nonexistent section is a no-op
        {
            let mut ini_modified = ini.clone();
            let indices = ini_modified.remove_sections("Section 3");
            assert!(indices.is_empty());
            
            let actual_sections = ini_modified.iter_sections().map(|section| section.key());
            let expected_sections = ini_modified.iter_sections().map(|section| section.key());
            assert_eq_iter!(actual_sections, expected_sections);
            
            assert_eq!(ini_modified.section_map, ini.section_map);
        }
    }
    
    #[test]
    fn ini_rename_section_at_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        {
            let mut ini = ini.clone();
            ini.rename_section_at(0, "secTION 1");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "secTION 1",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "SECTION 2",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        {
            let mut ini = ini.clone();
            ini.rename_section_at(5, "Section 3");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "Section 3",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
    }
    
    #[test]
    fn ini_rename_sections_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        {
            let mut ini = ini.clone();
            ini.rename_sections("secTION 0", "secTION 1");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "secTION 1",
                "Section 1",
                "Section 2",
                "secTION 1",
                "SECTION 1",
                "SECTION 2",
                "secTION 1",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        {
            let mut ini = ini.clone();
            ini.rename_sections("secTION 0", "Section 3");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 3",
                "Section 1",
                "Section 2",
                "Section 3",
                "SECTION 1",
                "SECTION 2",
                "Section 3",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
        {
            let mut ini = ini.clone();
            ini.rename_section_at(5, "Section 3");
            
            let actual_sections = ini.iter_sections().map(|section| section.key());
            let expected_sections = [
                "Section 0",
                "Section 1",
                "Section 2",
                "SECTION 0",
                "SECTION 1",
                "Section 3",
                "section 0",
                "section 1",
                "section 2",
            ];
            assert_eq_iter!(actual_sections, expected_sections);
            
            let mut expected_map = SectionMap::new();
            expected_map.rebuild(&ini.sections, ini.source());
            assert_eq!(ini.section_map, expected_map);
        }
    }
    
    #[test]
    fn ini_clear_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        ini.clear();
        
        assert!(ini.sections.is_empty());
        assert_eq_iter!(ini.global_section.iter_items(), &[]);
        assert_eq_iter!(ini.section_map.ordering().iter(), &[] as &[String]);
    }
    
    #[test]
    fn ini_iter_sections_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::parse(SOURCE);
        
        let actual_sections = ini.iter_sections().map(|section| section.key());
        let expected_sections = [
            "Section 0",
            "Section 1",
            "Section 2",
            "SECTION 0",
            "SECTION 1",
            "SECTION 2",
            "section 0",
            "section 1",
            "section 2",
        ];
        assert_eq_iter!(actual_sections, expected_sections);
    }
    
    #[test]
    fn ini_iter_sections_mut_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::parse(SOURCE);
        
        let actual_sections = ini.iter_sections_mut()
            .map(|section| section.key().to_owned());
        let expected_sections = [
            "Section 0",
            "Section 1",
            "Section 2",
            "SECTION 0",
            "SECTION 1",
            "SECTION 2",
            "section 0",
            "section 1",
            "section 2",
        ];
        assert_eq_iter!(actual_sections, expected_sections);
    }
}
