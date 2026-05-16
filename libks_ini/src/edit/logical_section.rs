use std::collections::HashSet;

use crate::item::Item;

use super::section::{SectionReader, SectionWriter};

#[derive(Debug)]
pub struct LogicalSection<'a> {
    sections: Vec<SectionReader<'a>>,
}

#[derive(Debug)]
pub struct LogicalSectionMut<'a> {
    sections: Vec<SectionWriter<'a>>,
}

impl<'a> LogicalSection<'a> {
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

    pub fn iter_props(&'a self) -> LogicalSectionPropsIter<'a, SectionReader<'a>> {
        LogicalSectionPropsIter::from(self)
    }
}

impl<'a> LogicalSectionMut<'a> {
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

    pub fn iter_props(&'a self) -> LogicalSectionPropsIter<'a, SectionWriter<'a>> {
        LogicalSectionPropsIter::from(self)
    }
}

#[allow(private_bounds)]
pub struct LogicalSectionPropsIter<'a, I>
where
    I: IterItems
{
    sections: std::iter::Rev<std::slice::Iter<'a, I>>,
    items: std::iter::Rev<std::slice::Iter<'a, Item>>,
    source: &'a str,
    keys_seen: HashSet<String>,
}

impl<'a> From<&'a LogicalSection<'a>> for LogicalSectionPropsIter<'a, SectionReader<'a>> {
    fn from(v_section: &'a LogicalSection<'a>) -> Self {
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

impl<'a> From<&'a LogicalSectionMut<'a>> for LogicalSectionPropsIter<'a, SectionWriter<'a>> {
    fn from(v_section: &'a LogicalSectionMut<'a>) -> Self {
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

impl<'a, I> Iterator for LogicalSectionPropsIter<'a, I>
where
    I: IterItems
{
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
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
        }
    }
}

trait IterItems {
    fn iter_items(&self) -> std::slice::Iter<'_, Item>;
}

impl<'a> IterItems for SectionReader<'a> {
    fn iter_items(&self) -> std::slice::Iter<'_, Item> {
        self.section.items.iter()
    }
}

impl<'a> IterItems for SectionWriter<'a> {
    fn iter_items(&self) -> std::slice::Iter<'_, Item> {
        self.section.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::test_macros::*;
    use crate::edit::Ini;
    
    #[test]
    fn logical_section_has_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::from(SOURCE);

        let section = ini.section("Section 0").unwrap();
        assert!(section.has("Prop 0"));
        assert!(section.has("Prop 1"));
        assert!(section.has("Prop 2"));
        assert!(!section.has("Prop 3"));
        
        let section = ini.section("Section 1").unwrap();
        assert!(section.has("Prop 0"));
        assert!(section.has("Prop 1"));
        assert!(section.has("Prop 2"));
        assert!(section.has("Prop 3"));
        assert!(section.has("Prop 4"));
        assert!(!section.has("Prop 5"));
        
        let section = ini.section("Section 2").unwrap();
        assert!(section.has("Prop 0"));
        assert!(section.has("Prop 1"));
        assert!(section.has("Prop 2"));
        assert!(!section.has("Prop 3"));
    }
    
    #[test]
    fn logical_section_mut_has_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::from(SOURCE);

        let section = ini.section_mut("Section 0").unwrap();
        assert!(section.has("Prop 0"));
        assert!(section.has("Prop 1"));
        assert!(section.has("Prop 2"));
        assert!(!section.has("Prop 3"));
        
        let section = ini.section_mut("Section 1").unwrap();
        assert!(section.has("Prop 0"));
        assert!(section.has("Prop 1"));
        assert!(section.has("Prop 2"));
        assert!(section.has("Prop 3"));
        assert!(section.has("Prop 4"));
        assert!(!section.has("Prop 5"));
        
        let section = ini.section_mut("Section 2").unwrap();
        assert!(section.has("Prop 0"));
        assert!(section.has("Prop 1"));
        assert!(section.has("Prop 2"));
        assert!(!section.has("Prop 3"));
    }
    
    #[test]
    fn logical_section_get_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::from(SOURCE);

        let section = ini.section("Section 0").unwrap();
        assert_eq!(section.get("Prop 0"), Some("Section 0/Prop 0/Value 2"));
        assert_eq!(section.get("Prop 1"), Some("Section 0/Prop 1/Value 1"));
        assert_eq!(section.get("Prop 2"), Some("Section 0/Prop 2/Value 0"));
        assert_eq!(section.get("Prop 3"), None);
        
        let section = ini.section("Section 1").unwrap();
        assert_eq!(section.get("Prop 0"), Some("Section 1/Prop 0/Value 0"));
        assert_eq!(section.get("Prop 1"), Some("Section 1/Prop 1/Value 0"));
        assert_eq!(section.get("Prop 2"), Some("Section 1/Prop 2/Value 0"));
        assert_eq!(section.get("Prop 3"), Some("Section 1/Prop 3/Value 0"));
        assert_eq!(section.get("Prop 4"), Some("Section 1/Prop 4/Value 0"));
        assert_eq!(section.get("Prop 5"), None);
        
        let section = ini.section("Section 2").unwrap();
        assert_eq!(section.get("Prop 0"), Some("Section 2/Prop 0/Value 5"));
        assert_eq!(section.get("Prop 1"), Some("Section 2/Prop 1/Value 3"));
        assert_eq!(section.get("Prop 2"), Some("Section 2/Prop 2/Value 1"));
        assert_eq!(section.get("Prop 3"), None);
    }
    
    #[test]
    fn logical_section_mut_get_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::from(SOURCE);

        let section = ini.section_mut("Section 0").unwrap();
        assert_eq!(section.get("Prop 0"), Some("Section 0/Prop 0/Value 2"));
        assert_eq!(section.get("Prop 1"), Some("Section 0/Prop 1/Value 1"));
        assert_eq!(section.get("Prop 2"), Some("Section 0/Prop 2/Value 0"));
        assert_eq!(section.get("Prop 3"), None);
        
        let section = ini.section_mut("Section 1").unwrap();
        assert_eq!(section.get("Prop 0"), Some("Section 1/Prop 0/Value 0"));
        assert_eq!(section.get("Prop 1"), Some("Section 1/Prop 1/Value 0"));
        assert_eq!(section.get("Prop 2"), Some("Section 1/Prop 2/Value 0"));
        assert_eq!(section.get("Prop 3"), Some("Section 1/Prop 3/Value 0"));
        assert_eq!(section.get("Prop 4"), Some("Section 1/Prop 4/Value 0"));
        assert_eq!(section.get("Prop 5"), None);
        
        let section = ini.section_mut("Section 2").unwrap();
        assert_eq!(section.get("Prop 0"), Some("Section 2/Prop 0/Value 5"));
        assert_eq!(section.get("Prop 1"), Some("Section 2/Prop 1/Value 3"));
        assert_eq!(section.get("Prop 2"), Some("Section 2/Prop 2/Value 1"));
        assert_eq!(section.get("Prop 3"), None);
    }
    
    #[test]
    fn logical_section_mut_set_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::from(SOURCE);

        let mut section = ini.section_mut("Section 0").unwrap();
        section.set("Prop 0", "Section 0/Prop 0/Value X");
        section.set("Prop 1", "Section 0/Prop 1/Value X");
        section.set("Prop 2", "Section 0/Prop 2/Value X");
        section.set("Prop 3", "Section 0/Prop 3/Value X");
        
        let mut section = ini.section_mut("Section 1").unwrap();
        section.set("Prop 0", "Section 1/Prop 0/Value X");
        section.set("Prop 1", "Section 1/Prop 1/Value X");
        section.set("Prop 2", "Section 1/Prop 2/Value X");
        section.set("Prop 3", "Section 1/Prop 3/Value X");
        section.set("Prop 4", "Section 1/Prop 4/Value X");
        section.set("Prop 5", "Section 1/Prop 5/Value X");
        
        let mut section = ini.section_mut("Section 2").unwrap();
        section.set("Prop 0", "Section 2/Prop 0/Value X");
        section.set("Prop 1", "Section 2/Prop 1/Value X");
        section.set("Prop 2", "Section 2/Prop 2/Value X");
        section.set("Prop 3", "Section 2/Prop 3/Value X");
        
        assert_eq!(ini.to_string(), after!("logical_section_mut_set_works.ini"));
    }
    
    #[test]
    fn logical_section_mut_unset_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::from(SOURCE);

        let mut section = ini.section_mut("Section 0").unwrap();
        section.unset("Prop 0");
        
        let mut section = ini.section_mut("Section 1").unwrap();
        section.unset("Prop 0");
        
        let mut section = ini.section_mut("Section 2").unwrap();
        section.unset("Prop 0");
        
        assert_eq!(ini.to_string(), after!("logical_section_mut_unset_works.ini"));
    }
    
    #[test]
    fn logical_section_iter_props_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let ini = Ini::from(SOURCE);
        
        let section = ini.section("Section 0").unwrap();
        let expected = HashMap::from([
            ("prop 0", "Section 0/Prop 0/Value 2"),
            ("PROP 1", "Section 0/Prop 1/Value 1"),
            ("Prop 2", "Section 0/Prop 2/Value 0"),
        ]);
        let actual = HashMap::from_iter(section.iter_props());
        assert_eq!(actual, expected);
        
        let section = ini.section("Section 1").unwrap();
        let expected = HashMap::from([
            ("prop 4", "Section 1/Prop 4/Value 0"),
            ("PROP 3", "Section 1/Prop 3/Value 0"),
            ("Prop 2", "Section 1/Prop 2/Value 0"),
            ("Prop 1", "Section 1/Prop 1/Value 0"),
            ("Prop 0", "Section 1/Prop 0/Value 0"),
        ]);
        let actual = HashMap::from_iter(section.iter_props());
        assert_eq!(actual, expected);
        
        let section = ini.section("Section 2").unwrap();
        let expected = HashMap::from([
            ("prop 0", "Section 2/Prop 0/Value 5"),
            ("PROP 1", "Section 2/Prop 1/Value 3"),
            ("Prop 2", "Section 2/Prop 2/Value 1"),
        ]);
        let actual = HashMap::from_iter(section.iter_props());
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn logical_section_mut_iter_props_works() {
        const SOURCE: &'static str = before!("duplicates.ini");
        let mut ini = Ini::from(SOURCE);
        
        let section = ini.section_mut("Section 0").unwrap();
        let expected = HashMap::from([
            ("prop 0", "Section 0/Prop 0/Value 2"),
            ("PROP 1", "Section 0/Prop 1/Value 1"),
            ("Prop 2", "Section 0/Prop 2/Value 0"),
        ]);
        let actual = HashMap::from_iter(section.iter_props());
        assert_eq!(actual, expected);
        
        let section = ini.section_mut("Section 1").unwrap();
        let expected = HashMap::from([
            ("prop 4", "Section 1/Prop 4/Value 0"),
            ("PROP 3", "Section 1/Prop 3/Value 0"),
            ("Prop 2", "Section 1/Prop 2/Value 0"),
            ("Prop 1", "Section 1/Prop 1/Value 0"),
            ("Prop 0", "Section 1/Prop 0/Value 0"),
        ]);
        let actual = HashMap::from_iter(section.iter_props());
        assert_eq!(actual, expected);
        
        let section = ini.section_mut("Section 2").unwrap();
        let expected = HashMap::from([
            ("prop 0", "Section 2/Prop 0/Value 5"),
            ("PROP 1", "Section 2/Prop 1/Value 3"),
            ("Prop 2", "Section 2/Prop 2/Value 1"),
        ]);
        let actual = HashMap::from_iter(section.iter_props());
        assert_eq!(actual, expected);
    }
}
