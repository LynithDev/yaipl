use std::collections::HashMap;

use super::object::{Object, ObjectValue};

#[derive(Debug, PartialEq)]
pub struct Environment {
    pub var_store: HashMap<String, Object>,
    pub functions_store: HashMap<String, Object>,
    pub is_root_env: bool,
}

impl Environment {
    fn new_opt_root(root: bool) -> Environment {
        Environment {
            var_store: HashMap::new(),
            functions_store: HashMap::new(),
            is_root_env: root,
        }
    }

    pub fn new_root() -> Environment {
        Environment::new_opt_root(true)
    }

    pub fn new() -> Environment {
        Environment::new_opt_root(false)
    }

    pub fn set_var(&mut self, identifier: String, value: ObjectValue) -> Option<&Object> {
        let object = Object::new(value);

        self.var_store.insert(identifier.to_owned(), object);
        self.var_store.get(identifier.as_str())
    }

    pub fn get_var(&self, identifier: String) -> Option<&Object> {
        self.var_store.get(identifier.as_str())
    }

}
