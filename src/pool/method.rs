use crate::types::{Value as VkValue, Result as VkResult};
use serde_json::value::Value;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use std::sync::Arc;

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
    pub sender: oneshot::Sender<Result<VkResult<Value>, Arc<reqwest::Error>>>,
}

impl MethodWithSender {
    pub fn new(method: Method, oneshot_sender: oneshot::Sender<Result<VkResult<Value>, Arc<reqwest::Error>>>) -> MethodWithSender {
        MethodWithSender { method, sender: oneshot_sender }
    }
}