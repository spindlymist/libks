use crate::{
    item::{Item, Prop, Padding4},
    Cows
};

#[derive(Debug, Clone, Default)]
pub struct ConcreteSection<'a> {
    items: Vec<Item<'a>>,
}

impl<'a> ConcreteSection<'a> {
    pub fn new(header: Item<'a>) -> Self {
        let items = match header {
            Item::Section(_, _) => vec![header],
            _ => panic!("Section header item must be Section variant"),
        };

        Self { items }
    }

    pub fn new_global() -> Self {
        Self { items: Vec::new() }
    }

    pub fn key(&self) -> &str {
        match &self.items[0] {
            Item::Section(key, _) => key,
            _ => unreachable!(),
        }
    }

    pub fn set_key(&mut self, to_key: &str) {
        match &mut self.items[0] {
            Item::Section(key, _) => *key = to_key.to_owned().into(),
            _ => unreachable!(),
        }
    }

    pub fn push_item(&mut self, item: Item<'a>) {
        self.items.push(item);
    }

    fn find_prop(&self, key: &str) -> Option<&Prop<'a>> {
        for item in self.items.iter().rev() {
            if let Item::Property(prop, _) = item {
                if prop.key.eq_ignore_ascii_case(key) {
                    return Some(prop);
                }
            }
        }
        None
    }

    fn find_prop_mut(&mut self, key: &str) -> Option<&mut Prop<'a>> {
        for item in self.items.iter_mut().rev() {
            if let Item::Property(prop, _) = item {
                if prop.key.eq_ignore_ascii_case(key) {
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
            .map(|prop| prop.value.as_ref())
    }

    pub fn set(&mut self, key: &str, value: String) {
        if let Some(kvp) = self.find_prop_mut(key) {
            kvp.value = value.into();
        }
        else {
            let item = Item::Property(Prop {
                key: key.to_owned().into(),
                value: value.into(),
            }, Padding4("", "", "", ""));
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
                Item::Property(prop, _) => prop.key.eq_ignore_ascii_case(key),
                _ => true,
            })
            .cloned()
            .collect();
    }

    pub fn rename(&mut self, from_key: &str, to_key: &str) {
        self.remove(to_key);
        for item in &mut self.items {
            match item {
                Item::Property(prop, _) if prop.key.eq_ignore_ascii_case(from_key) => {
                    prop.key = Cows::from(to_key.to_owned());
                }
                _ => (),
            }
        }
    }
}

impl<'a> std::fmt::Display for ConcreteSection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.items {
            f.write_str(&item.to_string())?;
        }
        Ok(())
    }
}
