use tokio::sync::oneshot;
use crate::types::{Value as VkValue, Result as VkResult};
use serde_json::value::Value;
use super::{MethodWithSender, Method};

pub enum Message {
    NewMethod(MethodWithSender),
    NewExecute(Vec<Method>, Vec<oneshot::Sender<VkResult<Value>>>),
    Terminate,
}