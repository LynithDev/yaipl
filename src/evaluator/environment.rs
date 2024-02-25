use std::collections::HashMap;

use super::object::{Object, ObjectValue};

#[derive(Clone, Debug)]
pub struct Environment {
    pub parent: Option<Box<Environment>>,
    pub var_store: HashMap<String, Object>,
    pub functions_store: HashMap<String, Object>,
}

impl Environment {
    pub fn with_parent(parent: Box<Environment>) -> Environment {
        Environment {
            parent: Some(parent),
            var_store: HashMap::new(),
            functions_store: HashMap::new(),
        }
    }

    pub fn new() -> Environment {
        Environment {
            parent: None,
            var_store: HashMap::new(),
            functions_store: HashMap::new(),
        }
    }

    pub fn set_var(&mut self, identifier: String, value: ObjectValue) -> Option<&Object> {
        let object = Object::new(value);

        self.var_store.insert(identifier.to_owned(), object);
        self.var_store.get(identifier.as_str())
    }

    pub fn get_var(&self, identifier: String) -> Option<&Object> {
        if let Some(result) = self.var_store.get(identifier.as_str()) {
            return Some(result);
        }

        if let Some(parent) = &self.parent {
            return parent.get_var(identifier);
        }

        None
    }

    pub fn get_var_err(&self, identifier: String) -> Result<&Object, String> {
        match self.get_var(identifier.to_owned()) {
            Some(object) => Ok(object),
            None => Err(format!("Undefined variable {:?}", identifier))
        }
    }

    pub fn set_function(&mut self, identifier: String, value: ObjectValue) -> Option<&Object> {
        let object = Object::new(value);

        self.functions_store.insert(identifier.to_owned(), object);
        self.functions_store.get(identifier.as_str())
    }

    pub fn get_function(&self, identifier: String) -> Option<&Object> {
        if let Some(function) = self.functions_store.get(identifier.as_str()) {
            return Some(function);
        }

        if let Some(parent) = &self.parent {
            return parent.get_function(identifier);
        }

        None
    }

    pub fn get_function_err(&self, identifier: String) -> Result<&Object, String> {
        match self.get_function(identifier.to_owned()) {
            Some(object) => Ok(object),
            None => Err(format!("Undefined function {:?}", identifier))
        }
    }
}
