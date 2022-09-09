use crate::Result as VkResult;
use serde_json::value::Value;
use tokio::sync::oneshot;
use std::sync::Arc;

mod params;
pub use params::Params;

pub struct Method {
    pub name: String,
    pub params: Params,
}

impl Method {
    pub fn new(name: String, params: Params) -> Method {
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