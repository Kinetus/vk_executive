mod execute_manager;
mod instance;
mod message;
mod worker;

pub use instance::Instance;
use vk_method::Method;
use message::Message;

use crossbeam_channel::unbounded;
use tokio::sync::oneshot;

use std::iter::ExactSizeIterator;

use crate::Result;
use serde_json::value::Value;

use execute_manager::ExecuteManager;

use worker::Worker;

pub type Sender = oneshot::Sender<Result<Value>>;

pub struct InstancePool {
    sender: crossbeam_channel::Sender<Message>,
    workers: Vec<Worker>,
    execute_manager: ExecuteManager,
}

impl InstancePool {
    pub fn new<Instances>(instances: Instances) -> InstancePool
    where
        Instances: ExactSizeIterator<Item = Instance>
    {
        let mut workers = Vec::with_capacity(instances.len());
        let (sender, receiver) = unbounded();

        let (event_sender, event_receiver) = unbounded();

        for (index, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(
                index,
                instance,
                receiver.clone(),
                event_sender.clone(),
            ));
        }

        let execute_manager = ExecuteManager::new(event_receiver, sender.clone());

        InstancePool {
            workers,
            sender,
            execute_manager,
        }
    }

    pub async fn run(&self, method: Method) -> Result<Value> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        if self.sender.is_empty() {
            // ! 1 unnecessary method. Need fix
            self.sender
                .send(Message::NewMethod(
                    method,
                    oneshot_sender,
                ))
                .unwrap();
        } else {
            self.execute_manager.push(method, oneshot_sender)?;
        }

        oneshot_receiver.await.unwrap()
    }
}

impl Drop for InstancePool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        drop(&self.sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vk_method::{Params, PairsArray};
    use dotenv::dotenv;
    use std::env;

    use futures::future::join_all;

    //TODO make vk mock server
    fn get_users() -> Vec<Value> {
        return vec![
            serde_json::from_str(
                r#"{
                "id": 1,
                "first_name": "Pavel",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 2,
                "first_name": "Alexandra",
                "last_name": "Vladimirova",
                "is_closed": true,
                "can_access_closed": false
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 3,
                "first_name": "DELETED",
                "last_name": "",
                "deactivated": "deleted"
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 4,
                "first_name": "DELETED",
                "last_name": "",
                "deactivated": "deleted"
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 5,
                "first_name": "Ilya",
                "last_name": "Perekopsky",
                "is_closed": false,
                "can_access_closed": true
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 6,
                "first_name": "Nikolay",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 7,
                "first_name": "Alexey",
                "last_name": "Kobylyansky",
                "is_closed": true,
                "can_access_closed": false
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 8,
                "first_name": "Aki",
                "last_name": "Sepiashvili",
                "is_closed": false,
                "can_access_closed": true
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 9,
                "first_name": "Nastya",
                "last_name": "Vasilyeva",
                "is_closed": true,
                "can_access_closed": false
            }"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{
                "id": 10,
                "first_name": "Alexander",
                "last_name": "Kuznetsov",
                "is_closed": true,
                "can_access_closed": false
            }"#,
            )
            .unwrap(),
        ];
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ten_tasks_three_workers() {
        dotenv().ok();
        let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(3)).unwrap();

        let pool = InstancePool::new(instances.into_iter());

        let mut vec = Vec::new();

        for i in 1..11 {
            let params = Params::try_from(PairsArray([("user_id", i)])).unwrap();

            vec.push(pool.run(Method::new("users.get", params)));
        }

        let responses = join_all(vec).await;

        for (index, res) in responses.into_iter().enumerate() {
            let res: Vec<Value> = serde_json::from_value(res.unwrap()).unwrap();
            assert_eq!(res[0], get_users()[index]);
        }
    }
}
