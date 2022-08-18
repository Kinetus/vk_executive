use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ExecuteError {
    #[error("queue is empty")]
    EmptyQueue
}