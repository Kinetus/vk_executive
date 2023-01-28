use once_cell::sync::Lazy;
use serde_json::Value;

const USERS_JSON: &str = include_str!("users.json");

pub static USERS: Lazy<Vec<Value>> = Lazy::new(|| serde_json::from_str(USERS_JSON).unwrap());
