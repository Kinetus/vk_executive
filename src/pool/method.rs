use crate::types::{Value as VkValue, Result as VkResult};
use serde_json::value::Value;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub params: HashMap<String, VkValue>,
}

impl Method {
    pub fn new(name: String, params: HashMap<String, VkValue>) -> Method {
        Method { name, params }
    }
}

pub struct MethodWithSender {
    pub method: Method,
    pub sender: oneshot::Sender<VkResult<Value>>,
}

impl MethodWithSender {
    pub fn new(method: Method, oneshot_sender: oneshot::Sender<VkResult<Value>>) -> MethodWithSender {
        MethodWithSender { method, sender: oneshot_sender }
    }
}