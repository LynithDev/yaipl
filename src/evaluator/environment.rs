use super::object::Object;

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    name_store: Vec<&'a str>,
    value_store: Vec<Object>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            name_store: Vec::with_capacity(64),
            value_store: Vec::with_capacity(64),
        }
    }

    pub fn size(&self) -> usize {
        self.name_store.len()
    }

    pub fn truncate(&mut self, size: usize) {
        self.name_store.truncate(size);
        self.value_store.truncate(size);
    }

    pub fn set(&mut self, identifier: &'a str, object: Object) {
        if self.name_store.contains(&identifier) {
            if let Some(pos) = self.name_store.iter().rev().position(|name| *name == identifier) {
                let index = self.value_store.len() - 1 - pos;
                self.value_store[index] = object;
                return;
            }
        }

        self.name_store.push(identifier);
        self.value_store.push(object);
    }

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        if let Some(pos) = self.name_store.iter().rev().position(|name| name == identifier) {
            return Some(&self.value_store[self.value_store.len() - 1 - pos]);
        }

        None
    }
}
