use serde::{Deserialize, Serialize, Deserializer};
use std::collections::HashMap;

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Result<T> {
    response(T),
    error(Error),
}

impl<T> Result<T> {
    pub fn unwrap(self) -> T {
        match self {
            Result::response(response) => response,
            Result::error(_) => panic!("called `Result::unwrap()` on an `Error` value"),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Result<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Result::response(t) => write!(f, "Response: {t}"),
            Result::error(error) => write!(f, "{error}")
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Error {
    error_code: u16,
    error_msg: String,
    #[serde(deserialize_with = "hashmap_from_vector_of_pairs")]
    request_params: Option<HashMap<String, String>>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.request_params {
            Some(params) => write!(f, "Error {}: {}\nRequest params: {:#?}", self.error_code, self.error_msg, params),
            None => write!(f, "Error {}: {}", self.error_code, self.error_msg),
        }
        
    }
}

fn hashmap_from_vector_of_pairs<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Option<HashMap<String, String>>, D::Error> {
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