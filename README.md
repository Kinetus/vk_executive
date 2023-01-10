# Fast VK 🚀

Library for fast data collection from [VK](https://vk.com)

```toml
[dependencies]
fast_vk = "0.6"
```

# Example

```rust
use fast_vk::{Client, Instance};
use vk_method::{Method, Params};

#[tokio::main]
async fn main() {
    let token = ["1234567890abcdef1234567890abcdef1234567890abcdef"];
    let instances = Instance::from_tokens(token).unwrap();

    let pool = Client::from_instances(instances);

    let mut params = Params::new();
    params.insert("user_id", 1);

    let response = pool.method(Method::new(
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
