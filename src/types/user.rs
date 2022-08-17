//https://vk.com/dev/objects/user

use serde::{Deserialize, Serialize};
use super::value;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[allow(unused)]
pub struct MinUser {
    pub id: value::Integer,
    pub first_name: value::String,
    pub last_name: value::String,
    pub deactivated: Option<value::String>,
    pub is_closed: Option<value::Boolean>,
    pub can_access_closed: Option<value::Boolean>
}
