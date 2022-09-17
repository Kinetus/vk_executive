mod error;
pub use error::{Result, Error};
pub(crate) use error::VkResult;

mod pool;
pub use pool::{Instance, InstancePool};