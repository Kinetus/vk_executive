use crate::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::result::Result as StdResult;
use thiserror::Error as ThisError;

/// Struct only for parsing VK Result
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VkResult<T> {
    Response(T),
    Error(VkError),
}

impl<T> From<VkResult<T>> for StdResult<T, VkError> {
    fn from(value: VkResult<T>) -> Self {
        match value {
            VkResult::Response(response) => Ok(response),
            VkResult::Error(error) => Err(error),
        }
    }
}

impl<T> From<VkResult<T>> for StdResult<T, Error> {
    fn from(value: VkResult<T>) -> Self {
        match value {
            VkResult::Response(response) => Ok(response),
            VkResult::Error(error) => Err(Error::VK(error)),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for VkResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Response(t) => write!(f, "Response: {t}"),
            Self::Error(error) => write!(f, "{error}"),
        }
    }
}

/// Represents any valid VK Error
#[derive(Debug, ThisError, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct VkError {
    pub error_code: u16,
    pub error_msg: String,
    #[serde(default)]
    #[serde(deserialize_with = "params_from_pairs")]
    pub request_params: Option<HashMap<String, String>>,
}

impl std::fmt::Display for VkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.request_params {
            Some(params) => {
                write!(
                    f,
                    "Error {}: {}\nRequest params: {:#?}",
                    self.error_code, self.error_msg, params
                )
            }
            None => {
                write!(f, "Error {}: {}", self.error_code, self.error_msg)
            }
        }
    }
}

/// Serializes [`HashMap`] from sequence of objects with fields key and value
///
/// For example, serializes this json
/// ```javascript
/// [
///     { "key": "meaning_of_life", "value": 42 },
///     { "key": "test", 1}
/// ]
/// ```
/// into `HashMap { "meaning_of_life": 42, "test": 1 }`
pub fn params_from_pairs<'de, D>(d: D) -> StdResult<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<Pair> = Deserialize::deserialize(d)?;

    if s.is_empty() {
        return Ok(None);
    }

    let mut map = HashMap::with_capacity(s.len());

    for Pair { key, value } in s {
        map.insert(key, value);
    }

    Ok(Some(map))
}

/// Represents any Request Param
#[derive(Debug, Deserialize, Serialize)]
struct Pair {
    key: String,
    value: String,
}
