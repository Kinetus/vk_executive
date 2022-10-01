use crate::vk_error::VkError;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("VK Error({0})")]
    VK(VkError),
    #[error("Arc({0})")]
    Arc(Arc<Error>),
    #[error("Custom({0})")]
    Custom(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::Custom(error)
    }
}
