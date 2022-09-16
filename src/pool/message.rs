use tokio::sync::oneshot;
use crate::Result as VkResult;
use serde_json::value::Value;
use super::{MethodWithSender, Method};
use std::sync::Arc;

pub enum Message {
    NewMethod(MethodWithSender),
    NewExecute(Vec<Method>, Vec<oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>>),
    Terminate,
}