// use tokio::sync::oneshot;
// use serde::{Serialize, Deserialize};
mod types;
mod pool;
// use pool::{InstancePool, Instance};

// use std::{time::Duration, fmt::format};
// use reqwest;

// pub struct Wrapper<'a> {
//     pool: InstancePool,
//     client: reqwest::Client,
//     instances: Vec<Instance<'a>>,
// }

// impl<'a> Wrapper<'a> {
//     pub fn new(instances: Vec<Instance>) -> Wrapper {
//         Wrapper {
//             pool: InstancePool::new(&instances),
//             client: reqwest::Client::new(),
//             instances
//         }
//     }

//     pub async fn method(&self, method: &str) -> Result<reqwest::Response, reqwest::Error> {
//         let url = format!("https://api.vk.com/method/{}", method);

//         let req = self.client.post(url).header("Content-Length", 0).query(&[("access_token", "123")]).send();
//         let (s, r) = oneshot::channel();

//         self.pool.run(async move {
//             s.send(req.await);
//         });

//         r.await.unwrap()
//     }
// }

// #[derive(Debug, Deserialize)]
// struct VKResponse {
//     error: String
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
    
//     #[tokio::test]
//     async fn method() {
//         let instances = vec![Instance::new("token", 500), Instance::new("token2", 500)];
//         let wrapper = Wrapper::new(instances);

//         let res = wrapper.method("users.get").await.unwrap();
        
//         //let json = res.json::<VKResponse>().await;
//         println!("{:?}", res.text().await);
//     }
// }