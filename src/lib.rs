//! Implementation of [Vkontakte](https://vk.com) Client.
//! General purpose is perform as much as possible requests per second.
//! It uses smart compression [`vk_method::Method`]s into [execute](https://vk.com/dev/execute).
//! Also you can create client which uses multiple [`Config`]s with [`Client::from_configs`].
//! Each `Config` includes own token, http client, api version and so on.
//! 
//! By default, it provides relatively low-level [`Client::method`]
//! However, there is `thisvk` feature avaible.
//! Consider using it if you want call vk methods directly from [`Client`]. For details see [thisvk](https://docs.rs/thisvk/0/thisvk/).

mod vk_error;
pub use vk_error::VkError;
pub(crate) use vk_error::VkResult;

mod error;
pub use error::Error;
pub use error::Result;

pub mod config;
pub use config::Config;

mod client;
pub use client::Client;

pub use vk_method;
pub use vk_method::Method;
