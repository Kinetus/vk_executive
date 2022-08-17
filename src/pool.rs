mod execute_manager;
mod instance;
mod message;
mod method;
mod worker;

pub use instance::Instance;
pub use message::Message;
pub use method::{Method, MethodWithSender};

use crossbeam_channel::unbounded;
use tokio::sync::oneshot;

use crate::types::Result as VkResult;
use serde_json::value::Value;

use worker::Worker;
use execute_manager::ExecuteManager;

pub struct InstancePool {
    sender: crossbeam_channel::Sender<Message>,
    workers: Vec<Worker>,
    execute_manager: ExecuteManager,
}

impl InstancePool {
    pub fn new(instances: Vec<Instance<'static>>, new_client: fn() -> reqwest::Client) -> InstancePool {
        let mut workers = Vec::with_capacity(instances.len());
        let (sender, receiver) = unbounded();

        let (event_sender, event_receiver) = unbounded();

        for (index, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(
                index,
                instance,
                receiver.clone(),
                new_client(),
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

    pub fn from_instances(instances: Vec<Instance<'static>>) -> InstancePool {
        InstancePool::new(instances, reqwest::Client::new)
    }

    pub fn run(&self, method: Method) -> oneshot::Receiver<VkResult<Value>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        if self.sender.is_empty() {
            self.sender
                .send(Message::NewMethod(MethodWithSender::new(
                    method,
                    oneshot_sender,
                )))
                .unwrap();
        } else {
            self.execute_manager
                .push(MethodWithSender::new(method, oneshot_sender));
        }

        oneshot_receiver
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
    use dotenv::dotenv;
    use std::env;

    use crate::types::{MinUser, Result as VkResult, Value as VkValue};
    use std::collections::HashMap;

    use futures::future::join_all;

    //TODO make vk mock server
    fn get_users() -> Vec<MinUser> {
        return vec![
            MinUser {
                id: 1,
                first_name: String::from("Pavel"),
                last_name: String::from("Durov"),
                deactivated: None,
                is_closed: Some(false),
                can_access_closed: Some(true),
            },
            MinUser {
                id: 2,
                first_name: String::from("Alexandra"),
                last_name: String::from("Vladimirova"),
                deactivated: None,
                is_closed: Some(true),
                can_access_closed: Some(false),
            },
            MinUser {
                id: 3,
                first_name: String::from("DELETED"),
                last_name: String::from(""),
                deactivated: Some(String::from("deleted")),
                is_closed: None,
                can_access_closed: None,
            },
            MinUser {
                id: 4,
                first_name: String::from("DELETED"),
                last_name: String::from(""),
                deactivated: Some(String::from("deleted")),
                is_closed: None,
                can_access_closed: None,
            },
            MinUser {
                id: 5,
                first_name: String::from("Ilya"),
                last_name: String::from("Perekopsky"),
                deactivated: None,
                is_closed: Some(false),
                can_access_closed: Some(true),
            },
            MinUser {
                id: 6,
                first_name: String::from("Nikolay"),
                last_name: String::from("Durov"),
                deactivated: None,
                is_closed: Some(false),
                can_access_closed: Some(true),
            },
            MinUser {
                id: 7,
                first_name: String::from("Alexey"),
                last_name: String::from("Kobylyansky"),
                deactivated: None,
                is_closed: Some(true),
                can_access_closed: Some(false),
            },
            MinUser {
                id: 8,
                first_name: String::from("Aki"),
                last_name: String::from("Sepiashvili"),
                deactivated: None,
                is_closed: Some(false),
                can_access_closed: Some(true),
            },
            MinUser {
                id: 9,
                first_name: String::from("Nastya"),
                last_name: String::from("Vasilyeva"),
                deactivated: None,
                is_closed: Some(true),
                can_access_closed: Some(false),
            },
            MinUser {
                id: 10,
                first_name: String::from("Alexander"),
                last_name: String::from("Kuznetsov"),
                deactivated: None,
                is_closed: Some(true),
                can_access_closed: Some(false),
            },
        ];
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ten_tasks_three_workers() {
        dotenv().ok();
        let instances = Instance::from_tokens(3, env::var("tokens").unwrap().split(","));

        let pool = InstancePool::new(instances, reqwest::Client::new);

        let mut vec = Vec::new();

        for i in 1..11 {
            let mut params = HashMap::new();

            params.insert("user_id".to_string(), VkValue::Integer(i));

            vec.push(pool.run(Method {
                name: "users.get".to_string(),
                params,
            }));
        }

        let responses = join_all(vec).await;

        for (index, res) in responses.into_iter().enumerate() {
            match res.unwrap() {
                VkResult::response(response) => {
                    let users: Vec<MinUser> = serde_json::from_value(response).unwrap();
                    assert_eq!(users[0], get_users()[index]);
                }
                VkResult::error(error) => {
                    panic!("{:?}", error);
                }
            }
        }
    }
}
