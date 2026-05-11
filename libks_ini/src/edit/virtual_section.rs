use super::Section;

pub struct VirtualSection<'a> {
    key: String,
    sections: &'a [Section],
    indices: Vec<usize>,
    unexplored_range_end: usize,
}

impl<'a> VirtualSection<'a> {
    pub fn new<K>(key: K, sections: &'a [Section]) -> Option<Self>
    where
        K: AsRef<str> + Into<String>
    {
        let index = find_last_section(key.as_ref(), sections)?;
        Some(Self {
            key: key.into(),
            sections,
            indices: vec![index],
            unexplored_range_end: index,
        })
    }
    
    fn next_section(&mut self) -> Option<usize> {
        if self.unexplored_range_end == 0 {
            return None;
        }
        
        let unexplored = &self.sections[..self.unexplored_range_end];
        let index = find_last_section(&self.key, unexplored)?;
        self.indices.push(index);
        self.unexplored_range_end = index;
        
        Some(index)
    }
    
    pub fn has<K: AsRef<str>>(&mut self, key: K) -> bool {
        self.get(key).is_some()
    }
    
    pub fn get<K: AsRef<str>>(&mut self, key: K) -> Option<&str> {
        for &i in &self.indices {
            if let Some(val) = self.sections[i].get(key.as_ref()) {
                return Some(val);
            }
        }
        while let Some(i) = self.next_section() {
            if let Some(val) = self.sections[i].get(key.as_ref()) {
                return Some(val);
            }
        }
        None
    }
}

pub struct VirtualSectionMut<'a> {
    key: String,
    sections: &'a mut [Section],
    indices: Vec<usize>,
    unexplored_range_end: usize,
}

impl<'a> VirtualSectionMut<'a> {
    pub fn new<K>(key: K, sections: &'a mut [Section]) -> Option<Self>
    where
        K: AsRef<str> + Into<String>
    {
        let i = find_last_section(key.as_ref(), sections)?;
        Some(Self {
            key: key.into(),
            sections,
            indices: vec![i],
            unexplored_range_end: i,
        })
    }
    
    fn next_section(&self) -> Option<usize> {
        let unexplored = &self.sections[..self.unexplored_range_end];
        if unexplored.is_empty() {
            return None;
        }
        let index = find_last_section(&self.key, unexplored)?;
        Some(index)
    }
    
    pub fn has<K: AsRef<str>>(&mut self, key: K) -> bool {
        self.get(key).is_some()
    }
    
    pub fn get<K: AsRef<str>>(&mut self, key: K) -> Option<&str> {
        for &i in &self.indices {
            if let Some(val) = self.sections[i].get(key.as_ref()) {
                return Some(val);
            }
        }
        while let Some(i) = self.next_section() {
            // Borrow checker doesn't like this in next_section, so we do it here
            self.indices.push(i);
            self.unexplored_range_end = i;
            
            if let Some(val) = self.sections[i].get(key.as_ref()) {
                return Some(val);
            }
        }
        None
    }
    
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str> + Into<String>,
        V: Into<String>,
    {
        let i = self.indices[0];
        self.sections[i].set(key, value);
    }
    
    pub fn remove<K: AsRef<str>>(&mut self, key: K) {
        for &i in &self.indices {
            self.sections[i].remove(key.as_ref());
        }
        while let Some(i) = self.next_section() {
            // Borrow checker doesn't like this in next_section, so we do it here
            self.indices.push(i);
            self.unexplored_range_end = i;
            
            self.sections[i].remove(key.as_ref());
        }
    }
}

fn find_last_section(key: &str, sections: &[Section]) -> Option<usize> {
    for i in (0..sections.len()).rev() {
        if sections[i].key().eq_ignore_ascii_case(key.as_ref()) {
            return Some(i);
        }
    }
    
    None
}
