use tokio::sync::oneshot;
use crate::types::Result as VkResult;
use serde_json::value::Value;
use super::{MethodWithSender, Method};
use std::sync::Arc;

pub enum Message {
    NewMethod(MethodWithSender),
    NewExecute(Vec<Method>, Vec<oneshot::Sender<Result<VkResult<Value>, Arc<reqwest::Error>>>>),
    Terminate,
}