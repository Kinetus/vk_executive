use crate::{Error as VkError, Result as VkResult};
use serde_json::value::Value;

use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use super::execute_manager::Event;
use super::{Instance, Message, Method, MethodWithSender, ExecuteCompiler};

pub struct Worker {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(
        id: usize,
        instance: Instance,
        receiver: crossbeam_channel::Receiver<Message>,
        event_sender: crossbeam_channel::Sender<Event>,
    ) -> Worker {
        let thread = tokio::spawn(async move {
            loop {
                event_sender.send(Event::FreeWorker).unwrap();

                match receiver.recv() {
                    Ok(message) => match message {
                        Message::NewMethod(MethodWithSender { method, sender }) => {
                            Worker::handle_method(method, sender, &instance)
                        }
                        Message::NewExecute(methods, senders) => {
                            Worker::handle_execute(methods, senders, &instance)
                        }
                        Message::Terminate => {
                            break;
                        }
                    },
                    Err(e) => {
                        panic!("{e}");
                    }
                }

                sleep(instance.time_between_requests).await;
            }
        });

        Worker {
            thread,
            id,
        }
    }

    fn handle_method(
        method: Method,
        sender: oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>,
        instance: &Instance,
    ) {
        let url = format!("{}/method/{}", &instance.api_url, method.name);
        let req = instance.client
            .post(url)
            .header("Content-Length", 0)
            .query(&method.params)
            .query(&[
                ("access_token", &instance.token),
            ])
            .query(&[
                ("v", &instance.api_version),
            ])
            .send();

        tokio::spawn(async move {
            let response = req.await;

            let resp = match response {
                Ok(response) => match response.json().await {
                    Ok(json) => Ok(json),
                    Err(error) => {
                        Err(Arc::new(error.into()))
                    }
                },
                Err(error) => Err(Arc::new(error.into())),
            };

            sender
                .send(resp)
                .unwrap();
        });
    }

    fn handle_execute(
        methods: Vec<Method>,
        senders: Vec<oneshot::Sender<Result<VkResult<Value>, Arc<anyhow::Error>>>>,
        instance: &Instance,
    ) {
        let execute = ExecuteCompiler::compile(methods);

        let url = format!("{}/method/execute", &instance.api_url);
        let req = instance.client
            .post(url)
            .header("Content-Length", 0)
            .query(&[("code", execute)])
            .query(&[("access_token", &instance.token)])
            .query(&[("v", "5.103")])
            .send();

        tokio::spawn(async move {
            let mut raw_response = match req.await {
                Ok(response) => response.json().await.unwrap(),
                Err(error) => {
                    let error = Arc::new(error.into());

                    for sender in senders {
                        sender.send(Err(Arc::clone(&error))).unwrap();
                    }

                    return;
                }
            };

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
                VkResult::Response(responses) => {
                    let responses: Vec<Value> = serde_json::from_value(responses).unwrap();

                    for (sender, response) in senders.into_iter().zip(responses.into_iter()) {
                        if let Some(bool) = response.as_bool() {
                            if bool == false {
                                sender
                                    .send(Ok(VkResult::Error(execute_errors.remove(0))))
                                    .unwrap();
                            }
                        } else {
                            sender.send(Ok(VkResult::Response(response))).unwrap();
                        }
                    }
                }
                VkResult::Error(error) => {
                    for sender in senders {
                        sender.send(Ok(VkResult::Error(error.clone()))).unwrap();
                    }
                }
            }
        });
    }
}
