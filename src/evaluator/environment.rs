use super::object::Object;

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub name_store: Vec<&'a str>,
    pub value_store: Vec<Object>,
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

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        if let Some(pos) = self.name_store.iter().rev().position(|name| name == identifier) {
            return Some(&self.value_store[self.value_store.len() - 1 - pos]);
        }

        None
    }

    pub fn get_func(&self, identifier: &String) -> Option<&Object> {
        self.get(&format!("__f_{}", identifier))
    }

    pub fn get_result(&self, identifier: &String) -> Result<&Object, String> {
        match self.get(identifier) {
            Some(object) => Ok(object),
            None => Err(format!("Undefined variable {:?}", identifier))
        }
    }

    pub fn get_func_result(&self, identifier: &String) -> Result<&Object, String> {
        match self.get_func(identifier) {
            Some(object) => Ok(object),
            None => Err(format!("Undefined function {:?}", identifier))
        }
    }
}
