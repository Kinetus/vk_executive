use super::{Sender, Method};

pub enum Message {
    NewMethod(Method, Sender),
    NewExecute(Vec<Method>, Vec<Sender>),
    Terminate,
}