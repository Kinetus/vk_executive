# Fast VK 🚀

Kernel for fast data collection from [VK](https://vk.com)

```toml
[dependencies]
fast_vk = "0.5"
```

# Example
```rust
use fast_vk::{InstancePool, Instance};
use std::collections::HashMap;
use vk_method::Method;

#[tokio::main]
async fn main() {
    let token = ["1234567890abcdef1234567890abcdef1234567890abcdef"];
    let instances = Instance::from_tokens(token).unwrap();

    let pool = InstancePool::from_instances(instances);

    let mut params = Params::new();
    params.insert("user_id", 1);

    let response = pool.run(Method::new(
        "users.get",
        params,
    )).await.unwrap();

    assert_eq!(
        response,
        serde_json::json!([
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
