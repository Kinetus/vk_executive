use tokio::sync::oneshot;
use crate::Result as VkResult;
use serde_json::value::Value;
use super::{Sender, Method};
use std::sync::Arc;

pub enum Message {
    NewMethod(Method, Sender),
    NewExecute(Vec<Method>, Vec<Sender>),
    Terminate,
}