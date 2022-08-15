use crossbeam_channel::unbounded;
use tokio::sync::oneshot;

use itertools::izip;

use crate::types::{Error as VkError, Result as VkResult};
use serde_json::value::Value;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::error::Error;

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod execute_manager;
mod instance;
mod message;
mod method;

use execute_manager::{Event, ExecuteManager};
pub use instance::Instance;
pub use message::Message;
pub use method::{Method, MethodWithSender};

pub struct InstancePool {
    sender: crossbeam_channel::Sender<Message>,
    workers: Vec<Worker>,
    execute_manager: ExecuteManager,
}

impl InstancePool {
    pub fn new(instances: Vec<Instance>, new_client: fn() -> reqwest::Client) -> InstancePool {
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

    pub fn from_instances(instances: Vec<Instance>) -> InstancePool {
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

struct Worker {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: Option<JoinHandle<()>>
}

impl Worker {
    pub fn new(
        id: usize,
        instance: Instance,
        receiver: crossbeam_channel::Receiver<Message>,
        client: reqwest::Client,
        event_sender: crossbeam_channel::Sender<Event>,
    ) -> Worker {

        let thread = tokio::spawn(async move {
            loop {
                event_sender.send(Event::FreeWorker).unwrap();

                match receiver.recv() {
                    Ok(message) => {
                        match message {
                            Message::NewMethod(MethodWithSender {
                                method,
                                sender: oneshot_sender,
                            }) => {
                                let url = format!("https://api.vk.com/method/{}", method.name);
                                let req = client
                                    .post(url)
                                    .header("Content-Length", 0)
                                    .query(&method.params)
                                    .query(&[
                                        ("access_token", instance.token()),
                                        ("v", "5.103".to_string()),
                                    ])
                                    .send();

                                
                                tokio::spawn(async move {
                                    oneshot_sender
                                        .send(req.await.unwrap().json().await.unwrap())
                                        .unwrap();
                                });
                            }
                            Message::NewExecute(methods, senders) => {
                                let execute = ExecuteManager::compile_execute(methods);

                                let url = format!("https://api.vk.com/method/execute");
                                let req = client
                                    .post(url)
                                    .header("Content-Length", 0)
                                    .query(&[
                                        ("code", execute)
                                    ])
                                    .query(&[
                                        ("access_token", instance.token()),
                                        ("v", "5.103".to_string()),
                                    ])
                                    .send();

                                tokio::spawn(async move {
                                    let mut raw_response: Value = req.await.unwrap().json().await.unwrap();

                                    let execute_errors_raw = if let Value::Object(ref mut map) = raw_response {
                                        map.remove("execute_errors")
                                    } else {
                                        None
                                    };
                                    
                                    let mut execute_errors: Vec<VkError> = Vec::new();

                                    if let Some(execute_errors_value) = execute_errors_raw {
                                        execute_errors = serde_json::from_value(execute_errors_value).unwrap();
                                    }
                                    
                                    let response: VkResult<Value> = serde_json::from_value(raw_response).unwrap();

                                    match response {
                                        VkResult::response(responses) => {
                                            let responses: Vec<Value> = serde_json::from_value(responses).unwrap();

                                            for (sender, response) in izip!(senders, responses) {
                                                if let Some(bool) = response.as_bool() {
                                                    if bool == false {
                                                        sender.send(VkResult::error(execute_errors.remove(0))).unwrap();
                                                    }
                                                } else {
                                                    sender.send(VkResult::response(response)).unwrap();
                                                }
                                            }
                                        }
                                        VkResult::error(error) => {
                                            for sender in senders {
                                                sender.send(VkResult::error(error.clone())).unwrap();
                                            }
                                        }
                                    }
                                });
                            }
                            Message::Terminate => {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        panic!("{e}");
                    }
                }

                sleep(Duration::from_millis(instance.millis_between_requests())).await;
            }
        });

        Worker {
            thread: Some(thread),
            id,
        }
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

    use crossbeam_channel::unbounded;
    use tokio::sync::oneshot;

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
        let instances = Instance::vector_from_args(3, env::var("tokens").unwrap().split(","), 500);

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

        let ress = join_all(vec).await;

        for (index, res) in ress.into_iter().enumerate() {
            if let VkResult::response(response) = res.unwrap() {
                let users: Vec<MinUser> = serde_json::from_value(response).unwrap();
                assert_eq!(users[0], get_users()[index]);
            }
        }
    }

    // mod worker {
    //     use super::*;

    //     #[tokio::test]
    //     async fn one_worker() {
    //         dotenv().ok();

    //         let (pool_sender, pool_receiver) = unbounded();

    //         let tokens = env::var("tokens").unwrap();
    //         let instance = Instance::new(tokens.split(",").next().unwrap().to_string(), 500);

    //         Worker::new(1, instance, pool_receiver.clone());

    //         let (oneshot_sender, oneshot_reciever) = oneshot::channel();

    //         let mut params = HashMap::new();

    //         params.insert("user_id".to_string(), Value::Integer(1));

    //         let task = Message::NewMethod {
    //             method: Method::new("users.get".to_string(), params),
    //             oneshot_sender,
    //         };

    //         pool_sender.send(task);

    //         let response = oneshot_reciever
    //             .await
    //             .unwrap()
    //             .unwrap()
    //             .json::<VkResult<Vec<MinUser>>>()
    //             .await;

    //         assert_eq!(
    //             response.unwrap(),
    //             VkResult::response(vec!(MinUser {
    //                 id: 1,
    //                 first_name: "Pavel".to_string(),
    //                 last_name: "Durov".to_string(),
    //                 deactivated: None,
    //                 is_closed: Some(false),
    //                 can_access_closed: Some(true)
    //             }))
    //         );
    //     }

    //     #[tokio::test]
    //     async fn five_workers_ten_tasks() {
    //         dotenv().ok();

    //         let (pool_sender, pool_receiver) = unbounded();
    //         let mut workers = Vec::new();

    //         let instances = Instance::vector_from_args(5, env::var("tokens").unwrap().split(","), 500);

    //         for (i, instance) in instances.into_iter().enumerate() {
    //             workers.push(Worker::new(i, instance, pool_receiver.clone()));
    //         }

    //         let mut oneshot_recievers = Vec::new();

    //         for i in 1..11 {
    //             let (oneshot_sender, oneshot_reciever) = oneshot::channel();
    //             oneshot_recievers.push(oneshot_reciever);

    //             let mut params = HashMap::new();
    //             params.insert("user_id".to_string(), Value::Integer(i));

    //             {
    //                 let task = Message::NewMethod { method: Method::new("users.get".to_string(), params), oneshot_sender };

    //                 pool_sender.send(task);
    //             }
    //         }

    //         for (i, oneshot_reciever) in oneshot_recievers.into_iter().enumerate() {
    //             println!("{:?}", oneshot_reciever.await.unwrap().unwrap().json::<VkResult<Vec<MinUser>>>().await.unwrap());
    //         }
    //     }
    // }
}
