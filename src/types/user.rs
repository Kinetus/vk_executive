use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[allow(unused)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub deactivated: Option<String>,
    pub is_closed: Option<bool>,
    pub can_access_closed: Option<bool>
} //https://vk.com/dev/objects/user

// impl User {
//     pub fn new(id: i32, first_name: String, last_name: String, is_closed: bool, can_access_closed: bool) -> User {
//         User { id, first_name, last_name, is_closed, can_access_closed }
//     }
// }