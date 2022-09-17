use crate::Result as VkResult;
use serde_json::value::Value;
use tokio::sync::oneshot;
use std::sync::Arc;
pub use vk_method::{Method, Params};

pub struct MethodWithSender {
    pub method: Method,
    pub sender: oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>,
}

impl MethodWithSender {
    pub fn new(method: Method, oneshot_sender: oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>) -> MethodWithSender {
        MethodWithSender { method, sender: oneshot_sender }
    }
}