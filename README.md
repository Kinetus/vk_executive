# Fast VK 🚀

Kernel for fast collection data from [VK](https://vk.com)

```toml
[dependencies]
fast_vk = "0.2"
```

# Example
```rust
use fast_vk::{InstancePool, Instance};
use std::collections::HashMap;
use vk_method::Method;
use serde_json::json;

#[tokio::main]
async fn main() {
    let token = ["1234567890abcdef1234567890abcdef1234567890abcdef"].into_iter();
    let instances = Instance::from_tokens(token).unwrap();

    let pool = InstancePool::new(instances.into_iter());

    let mut params = HashMap::new();

    params.insert("user_id".to_string(), "1".to_string());
    
    let response = pool.run(Method::new(
        "users.get",
        params.into(),
    )).await.unwrap();

    assert_eq!(
        response,
        json!([
            {
                "id": 1,
                "first_name": "Pavel",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }
        ])
    )
}
```