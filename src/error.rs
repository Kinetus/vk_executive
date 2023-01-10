use crate::vk_error::VkError;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

/// The Errors that may occur when processing a Method.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Represents any VK Error
    #[error("VK Error({0})")]
    VK(VkError),
    /// Represents [`Error`] inside Arc
    #[error("Arc({0})")]
    Arc(Arc<Error>),
    /// Represents any Error
    #[error("Custom({0})")]
    Custom(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::Custom(error)
    }
}
