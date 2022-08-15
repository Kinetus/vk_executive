use itertools::izip;

use crate::types::{Error as VkError, Result as VkResult};
use serde_json::value::Value;

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use super::execute_manager::{ExecuteManager, Event};
use super::{Instance, Message, MethodWithSender};

pub struct Worker {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: Option<JoinHandle<()>>,
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
                    Ok(message) => match message {
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
                                .query(&[("code", execute)])
                                .query(&[
                                    ("access_token", instance.token()),
                                    ("v", "5.103".to_string()),
                                ])
                                .send();

                            tokio::spawn(async move {
                                let mut raw_response: Value =
                                    req.await.unwrap().json().await.unwrap();

                                let execute_errors_raw =
                                    if let Value::Object(ref mut map) = raw_response {
                                        map.remove("execute_errors")
                                    } else {
                                        None
                                    };

                                let mut execute_errors: Vec<VkError> = Vec::new();

                                if let Some(execute_errors_value) = execute_errors_raw {
                                    execute_errors =
                                        serde_json::from_value(execute_errors_value).unwrap();
                                }

                                let response: VkResult<Value> =
                                    serde_json::from_value(raw_response).unwrap();

                                match response {
                                    VkResult::response(responses) => {
                                        let responses: Vec<Value> =
                                            serde_json::from_value(responses).unwrap();

                                        for (sender, response) in izip!(senders, responses) {
                                            if let Some(bool) = response.as_bool() {
                                                if bool == false {
                                                    sender
                                                        .send(VkResult::error(
                                                            execute_errors.remove(0),
                                                        ))
                                                        .unwrap();
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
                    },
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
