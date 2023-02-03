# VK Executive 🚀

Library for fast data collection from [VK](https://vk.com)

```toml
[dependencies]
vk_executive = "0.7"
```

# Example

```rust
use vk_executive::{Client, Instance};
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
