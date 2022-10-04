//! Library for fast data collection from [VK](https://vk.com)
//! 
//! By default, it provides relatively low-level [`Client::method`]
//! However, there is `thisvk` feature avaible.
//! With this feature you can call vk methods directly from [`Client`]. For details see [thisvk](https://docs.rs/thisvk/0/thisvk/).

mod vk_error;
pub use vk_error::VkError;
pub(crate) use vk_error::VkResult;

mod error;
pub use error::Error;
pub(crate) use error::Result;

pub mod instance;
pub use instance::Instance;

mod client;
pub use client::Client;

pub use vk_method;