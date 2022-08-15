use serde::{Deserialize, Serialize, Deserializer};
use std::collections::HashMap;

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Result<T> {
    response(T),
    error(Error),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Error {
    error_code: u16,
    error_msg: String,
    #[serde(deserialize_with = "hashmap_from_vector_of_pairs")]
    request_params: Option<HashMap<String, String>>,
}

impl<T> Result<T> {
    pub fn unwrap(self) -> T {
        match self {
            Result::response(response) => response,
            Result::error(_) => panic!("called `Result::unwrap()` on an `Error` value"),
        }
    }
}

fn hashmap_from_vector_of_pairs<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<HashMap<String, String>>, D::Error> {
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
