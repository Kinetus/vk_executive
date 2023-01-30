use super::{ResultSender, Method};

/// Message that sends to [`Worker`]
#[derive(Debug)]
pub enum Message {
    NewMethod(Method, ResultSender),
}
