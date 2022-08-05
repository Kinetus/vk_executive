use serde::{Deserialize, Serialize};
use super::vk_type;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[allow(unused)]
pub struct MinUser {
    pub id: vk_type::Integer,
    pub first_name: vk_type::String,
    pub last_name: vk_type::String,
    pub deactivated: Option<vk_type::String>,
    pub is_closed: Option<vk_type::Boolean>,
    pub can_access_closed: Option<vk_type::Boolean>
} //https://vk.com/dev/objects/user

// impl User {
//     pub fn new(id: i32, first_name: String, last_name: String, is_closed: bool, can_access_closed: bool) -> User {
//         User { id, first_name, last_name, is_closed, can_access_closed }
//     }
// }