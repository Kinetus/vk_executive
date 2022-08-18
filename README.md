# Fast VK 🚀

Kernel for fast collection data from [VK](https://vk.com)

```toml
[dependencies]
fast_vk = "1.0"
```

# Example
```rust
use fast_vk::{InstancePool, Instance, Method, Value, Result, MinUser};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let token = ["1234567890abcdef1234567890abcdef1234567890abcdef"].into_iter();
    let instances = Instance::from_tokens(token);

    let pool = InstancePool::new(instances, reqwest::Client::new);

    let mut params = HashMap::new();

    params.insert("user_id".to_string(), Value::Integer(1));

    let response: Result<Vec<MinUser>> = pool.run(Method {
        name: "users.get".to_string(),
        params,
    }).await.json();

    println!("{:?}", response);
}
```
