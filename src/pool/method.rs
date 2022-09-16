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
    pub fn new<T: ToString>(name: T, params: Params) -> Method {
        Method { name: name.to_string(), params }
    }
}

pub struct MethodWithSender {
    pub method: Method,
    pub sender: oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>,
}

impl MethodWithSender {
    pub fn new(method: Method, oneshot_sender: oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>) -> MethodWithSender {
        MethodWithSender { method, sender: oneshot_sender }
    }
}