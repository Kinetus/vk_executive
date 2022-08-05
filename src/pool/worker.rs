use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod instance;
mod method;
mod state;
mod message;

pub use instance::Instance;
pub use method::Method;
pub use state::State;
pub use message::Message;

pub struct Worker {
    id: usize,
    thread: JoinHandle<()>,
    state: State
}

impl Worker {
    pub fn new(
        id: usize,
        instance: Instance,
        receiver: crossbeam_channel::Receiver<Message>,
    ) -> Worker {
        let client = reqwest::Client::new();

        let thread = tokio::spawn(async move {
            loop {
                println!("new iter in {}", id);

                match receiver.try_recv() {
                    Ok(message) => match message {
                        Message::NewTask {
                            method,
                            oneshot_sender,
                        } => {
                            println!("Worker {} got a task; executing.", id);

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
                                oneshot_sender.send(req.await);
                            });
                        }
                        Message::Terminate => {
                            println!("Worker {} got a terminate message; terminating.", id);

                            break;
                        }
                    },
                    Err(_) => {
                        println!("empty channel")
                    }
                }

                sleep(Duration::from_millis(instance.millis_between_requests())).await;
            }
        });

        Worker { thread, id, state: State::Sleeping }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, vec};

    use super::*;
    use crate::types::{MinUser, VKResult, Value};
    use crossbeam_channel::unbounded;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn one_worker() {
        dotenv().ok();
        
        let (pool_sender, pool_receiver) = unbounded();

        let tokens = env::var("tokens").unwrap();
        let instance = Instance::new(tokens.split(",").next().unwrap().to_string(), 500);

        Worker::new(1, instance, pool_receiver.clone());

        let (oneshot_sender, oneshot_reciever) = oneshot::channel();

        let mut params = HashMap::new();

        params.insert("user_id".to_string(), Value::Integer(1));

        let task = Message::NewTask {
            method: Method::new("users.get".to_string(), params),
            oneshot_sender,
        };

        pool_sender.send(task);
        
        let response = oneshot_reciever
            .await
            .unwrap()
            .unwrap()
            .json::<VKResult<Vec<MinUser>>>()
            .await;
        
        assert_eq!(
            response.unwrap(),
            VKResult::response(vec!(MinUser {
                id: 1,
                first_name: "Pavel".to_string(),
                last_name: "Durov".to_string(),
                deactivated: None,
                is_closed: Some(false),
                can_access_closed: Some(true)
            }))
        );
    }
    
    #[tokio::test]
    async fn five_workers_ten_tasks() {
        dotenv().ok();

        let (pool_sender, pool_receiver) = unbounded();
        let mut workers = Vec::new();
        
        let instances = Instance::vector_from_args(5, env::var("tokens").unwrap().split(","), 500);

        for (i, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(i, instance, pool_receiver.clone()));
        }

        let mut oneshot_recievers = Vec::new();

        for i in 1..11 {
            let (oneshot_sender, oneshot_reciever) = oneshot::channel();
            oneshot_recievers.push(oneshot_reciever);

            let mut params = HashMap::new();
            params.insert("user_id".to_string(), Value::Integer(i));

            {
                let task = Message::NewTask { method: Method::new("users.get".to_string(), params), oneshot_sender };

                pool_sender.send(task);
            }
        }

        for (i, oneshot_reciever) in oneshot_recievers.into_iter().enumerate() {
            println!("{:?}", oneshot_reciever.await.unwrap().unwrap().json::<VKResult<Vec<MinUser>>>().await.unwrap());
        }
    }
}
