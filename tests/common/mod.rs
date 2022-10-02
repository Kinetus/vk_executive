use serde_json::Value;

const USERS_JSON: &str = include_str!("users.json");

lazy_static::lazy_static! {
    pub static ref USERS: Vec<Value> = serde_json::from_str(&USERS_JSON).unwrap();
}