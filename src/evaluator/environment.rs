use std::collections::HashMap;

use super::object::Object;

#[derive(Clone, Debug, PartialEq)]
pub struct Environment<'a> {
    pub globals_store: HashMap<String, Object<'a>>,
    pub functions_store: HashMap<String, Object<'a>>,
    pub is_root_env: bool,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            globals_store: HashMap::new(),
            functions_store: HashMap::new(),
            is_root_env: true,
        }
    }
}
