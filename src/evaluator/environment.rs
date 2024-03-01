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

    pub fn set(&mut self, identifier: &'a str, object: Object) {
        self.name_store.push(identifier);
        self.value_store.push(object);
    }

    pub fn get_by_index(&self, index: usize) -> Option<(String, Object)> {
        let name = self.name_store.get(index);
        let value = self.value_store.get(index);

        if name.is_some() && value.is_some() {
            return Some(((*name.expect("Couldn't take name")).to_owned(), value.expect("Couldn't take value").to_owned()));
        }

        None
    }

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        if let Some(pos) = self.name_store.iter().rev().position(|name| name == identifier) {
            return Some(&self.value_store[self.value_store.len() - 1 - pos]);
        }

        None
    }
}
