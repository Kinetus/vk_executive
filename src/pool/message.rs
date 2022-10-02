use super::{Sender, Method};

#[derive(Debug)]
pub enum Message {
    NewMethod(Method, Sender),
    Terminate,
}