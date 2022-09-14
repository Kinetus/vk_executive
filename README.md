# Fast VK 🚀

Kernel for fast collection data from [VK](https://vk.com)

```toml
[dependencies]
fast_vk = "0.1"
```

# Example
```rust
use fast_vk::{InstancePool, Instance, Method, Value as VkValue, Result};
use serde_json::value::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let token = ["1234567890abcdef1234567890abcdef1234567890abcdef"].into_iter();
    let instances = Instance::from_tokens(token).into_iter();

    let pool = InstancePool::new(instances);

    let mut params = HashMap::new();

    params.insert("user_id".to_string(), VkValue::Integer(1));

    let response: Result<Vec<Value>> = pool.run(Method {
        name: "users.get".to_string(),
        params,
    }).await.unwrap().json().unwrap();

    assert_eq!(
        response,
        Result::Response(vec![
            serde_json::from_str(r#"{
                "id": 1,
                "first_name": "Pavel",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }"#).unwrap()
        ])
    )
}
```