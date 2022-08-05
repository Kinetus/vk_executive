use crossbeam_channel::unbounded;
use tokio::sync::oneshot;
use futures::future::join_all;

mod worker;
use worker::{Worker, Message, Instance, Method};
pub struct InstancePool {
    sender: crossbeam_channel::Sender<Message>,
    workers: Vec<Worker>,
}

impl InstancePool {
    pub fn new(instances: Vec<Instance>) -> InstancePool {
        let mut workers = Vec::with_capacity(instances.len());
        let (sender, receiver) = unbounded();

        for (index, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(index, instance, receiver.clone()));
        }

        InstancePool { workers, sender }
    }

    pub fn run(&self, method: Method) -> oneshot::Receiver<Result<reqwest::Response, reqwest::Error>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        self.sender.send(Message::NewTask { method, oneshot_sender });

        oneshot_receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::collections::HashMap;
    use crate::types::{MinUser, VKResult, Value};
    use dotenv::dotenv;

    #[tokio::test]
    async fn ten_tasks_two_workers() {
        dotenv().ok();
        let instances = Instance::vector_from_args(2, env::var("tokens").unwrap().split(","), 500);

        let pool = InstancePool::new(instances);

        let mut vec = Vec::new();
        
        for i in 1..11 {
            let mut params = HashMap::new();

            params.insert("user_id".to_string(), Value::Integer(i));

            vec.push(pool.run(Method { name: "users.get".to_string(), params }));
        }

        let resps = join_all(vec).await;

        for resp in resps {
            println!("{:?}", resp.unwrap().unwrap().json::<VKResult<Vec<MinUser>>>().await.unwrap());
        }
    }
}

