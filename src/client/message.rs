use super::{Sender, Method};

/// Message that sends to [`Worker`]
#[derive(Debug)]
pub enum Message {
    NewMethod(Method, Sender),
    Terminate,
}