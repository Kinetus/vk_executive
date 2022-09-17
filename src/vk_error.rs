use serde::{Deserialize, Serialize, Deserializer};
use std::collections::HashMap;
use std::result::Result as StdResult;
use thiserror::Error as ThisError;

pub type Result<T> = StdResult<T, VkError>;

/// Use this only for parsing response from VK
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VkResult<T> {
    Response(T),
    Error(VkError),
}

impl<T> Into<Result<T>> for VkResult<T> {
    fn into(self) -> StdResult<T, VkError> {
        match self {
            VkResult::Response(response) => StdResult::Ok(response),
            VkResult::Error(error) => StdResult::Err(error)
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for VkResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            VkResult::Response(t) => write!(f, "Response: {t}"),
            VkResult::Error(error) => write!(f, "{error}")
        }
    }
}

#[derive(Debug, ThisError, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct VkError {
    error_code: u16,
    error_msg: String,
    #[serde(deserialize_with = "params_from_pairs")]
    request_params: Option<HashMap<String, String>>,
}

impl std::fmt::Display for VkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.request_params {
            Some(params) => {
                write!(
                    f,
                    "Error {}: {}\nRequest params: {:#?}",
                    self.error_code,
                    self.error_msg,
                    params
                )
            },
            None => {
                write!(
                    f,
                    "Error {}: {}",
                    self.error_code,
                    self.error_msg
                )
            }
        }
    }
}

fn params_from_pairs<'de, D>(d: D) -> StdResult<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>
{
    let s: Vec<Pair> = Deserialize::deserialize(d)?;

    if s.len() == 0 {
        return Ok(None);
    }

    let mut map = HashMap::with_capacity(s.len());

    for Pair { key, value } in s {
        map.insert(key, value);
    }

    Ok(Some(map))
}

#[derive(Debug, Deserialize, Serialize)]
struct Pair {
    key: String,
    value: String,
}