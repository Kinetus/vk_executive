mod vk_error;
pub use vk_error::VkError;
pub(crate) use vk_error::VkResult;

mod error;
pub use error::{Error, Result};

mod client;
pub use client::{Instance, Client};

pub use vk_method;