use crate::vk_error::VkError;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

/// The Errors that may occur when processing a Method.
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// Represents any VK error
    #[error("VK error({0})")]
    VK(VkError),
    /// Represents any shared VK error
    /// For example: [6] Too many requests
    #[error("Shared VK error({0})")]
    SharedVK(Arc<VkError>),
    /// Represents any network error
    #[error("Network error({0})")]
    Network(Arc<hyper::Error>),
    /// Represents any serialization error
    #[error("Serializion error({0})")]
    Serialization(Arc<serde_json::Error>),
}

impl From<VkError> for Error {
    fn from(error: VkError) -> Self {
        Self::VK(error)
    }
}

impl From<Arc<VkError>> for Error {
    fn from(error: Arc<VkError>) -> Self {
        Self::SharedVK(error) 
    }
}

impl From<Arc<hyper::Error>> for Error {
    fn from(error: Arc<hyper::Error>) -> Self {
        Self::Network(error) 
    }
}

impl From<Arc<serde_json::Error>> for Error {
    fn from(error: Arc<serde_json::Error>) -> Self {
        Self::Serialization(error)
    }
}
