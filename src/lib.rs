#[doc = include_str!("../README.md")]

mod types;
pub use types::{Value, Result, MinUser};

mod pool;
pub use pool::{Instance, Method, InstancePool};

