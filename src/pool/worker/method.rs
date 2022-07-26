use crate::types::Value;
use std::collections::HashMap;

pub struct Method {
    pub name: String,
    pub params: HashMap<String, Value>,
}

impl Method {
    pub fn new(name: String, params: HashMap<String, Value>) -> Method {
        Method { name, params }
    }
}
