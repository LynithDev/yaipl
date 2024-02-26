use super::object::{Object, ObjectValue};

#[derive(Clone, Debug)]
pub struct Environment {
    pub parent: Option<Box<Environment>>,
    pub name_store: Vec<String>,
    pub value_store: Vec<Object>,
}

impl Environment {
    pub fn with_parent(parent: Box<Environment>) -> Environment {
        Environment {
            parent: Some(parent),
            name_store: Vec::with_capacity(64),
            value_store: Vec::with_capacity(64),
        }
    }

    pub fn new() -> Environment {
        Environment {
            parent: None,
            name_store: Vec::with_capacity(64),
            value_store: Vec::with_capacity(64),
        }
    }
}

impl Environment {

    pub fn destroy(&mut self) {
        self.name_store.clear();
        self.value_store.clear();
    }

    pub fn set(&mut self, identifier: String, value: ObjectValue) {
        let object = Object::new(value);
        let name = match object.is_function() {
            true => format!("__f_{}", identifier),
            false => identifier
        };

        self.name_store.push(name);
        self.value_store.push(object);
    }

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        if let Some(pos) = self.name_store.iter().rev().position(|name| name == identifier) {
            return Some(&self.value_store[self.value_store.len() - 1 - pos]);
        }

        if let Some(parent) = &self.parent {
            return parent.get(identifier);
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
