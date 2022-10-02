use crate::{Error, VkError, VkResult};

use super::{Instance, Message, Method, Sender, TaskReceiver, MAX_METHODS_IN_EXECUTE};

use tokio::sync::mpsc;
use vk_execute_compiler::ExecuteCompiler;

use serde::Serialize;
use serde_json::value::Value;

use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

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
        receiver: TaskReceiver
    ) -> Worker {
        let thread = tokio::spawn(async move {
            'thread_loop: loop {
                let mut receiver = receiver.lock().await;

                match receiver.recv().await {
                    Some(message) => match message {
                        Message::NewMethod(method, sender) => {
                            
                            let mut methods: Vec<Method> = Vec::new();
                            let mut senders: Vec<Sender> = Vec::new();

                            //because first has already been received
                            'method_collection: for _ in 0..MAX_METHODS_IN_EXECUTE - 1 {
                                match receiver.try_recv() {
                                    Ok(message) => match message {
                                        Message::NewMethod(method, sender) => {
                                            methods.push(method);
                                            senders.push(sender)
                                        }
                                        Message::Terminate => break 'thread_loop,
                                    },
                                    Err(reason) => match reason {
                                        mpsc::error::TryRecvError::Empty => break 'method_collection,
                                        mpsc::error::TryRecvError::Disconnected => break 'thread_loop,
                                    }
                                }
                            }

                            if methods.len() == 0 {
                                Worker::handle_method(method, sender, &instance);
                            } else {
                                methods.push(method);
                                senders.push(sender);

                                Worker::handle_execute(methods, senders, &instance);
                            };
                        },
                        Message::Terminate => break,
                    },
                    None => {
                        break;
                    }
                }

                //important! Unlock mutex before sleep
                drop(receiver);
                sleep(instance.time_between_requests).await;
            }
        });

        Worker { thread, id }
    }

    fn handle_method(method: Method, sender: Sender, instance: &Instance) {
        let url = format!("{}/method/{}", &instance.api_url, method.name);
        let mut req = instance
            .client
            .post(url)
            .header("Content-Length", 0)
            // .query(&method.params)
            .query(&[("access_token", &instance.token)])
            .query(&[("v", &instance.api_version)])
            .build()
            .unwrap();

        {
            let mut pairs = req.url_mut().query_pairs_mut();
            let serializer = comma_serde_urlencoded::Serializer::new(&mut pairs);
            method.params.serialize(serializer).unwrap();
        }

        let req = instance.client.execute(req);

        tokio::spawn(async move {
            let response = req.await;

            let resp = match response {
                Ok(response) => match response.json::<VkResult<Value>>().await {
                    Ok(json) => json.into(),
                    Err(error) => Err(Error::Custom(error.into())),
                },
                Err(error) => Err(Error::Custom(error.into())),
            };

            sender.send(resp).unwrap();
        });
    }

    fn handle_execute(methods: Vec<Method>, senders: Vec<Sender>, instance: &Instance) {
        let execute = ExecuteCompiler::compile(methods);

        let url = format!("{}/method/execute", &instance.api_url);
        let req = instance
            .client
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
                    let error = Arc::new(Error::Custom(error.into()));

                    for sender in senders {
                        sender.send(Err(Error::Arc(Arc::clone(&error)))).unwrap();
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

            let response: Result<Value, VkError> =
                serde_json::from_value::<VkResult<Value>>(raw_response)
                    .unwrap()
                    .into();

            match response {
                Ok(responses) => {
                    let responses: Vec<Value> = serde_json::from_value(responses).unwrap();

                    for (sender, response) in senders.into_iter().zip(responses.into_iter()) {
                        if let Some(bool) = response.as_bool() {
                            if bool == false {
                                sender
                                    .send(Err(Error::VK(execute_errors.remove(0))))
                                    .unwrap();
                            }
                        } else {
                            sender.send(Ok(response)).unwrap();
                        }
                    }
                }
                Err(error) => {
                    let error = Arc::new(Error::VK(error));

                    for sender in senders {
                        sender.send(Err(Error::Arc(Arc::clone(&error)))).unwrap();
                    }
                }
            }
        });
    }
}
