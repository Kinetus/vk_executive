mod vk_error;
pub use vk_error::VkError;
pub(crate) use vk_error::VkResult;

mod error;
pub use error::{Error, Result};

pub mod instance;
pub use instance::Instance;

mod client;
pub use client::Client;

pub use vk_method;