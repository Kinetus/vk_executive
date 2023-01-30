use crate::{Error, VkError, VkResult};

use super::{Instance, Message, ResultSender, TaskReceiver, MAX_METHODS_IN_EXECUTE};

use tokio::sync::mpsc;
use vk_execute_compiler::ExecuteCompiler;

use serde::{Serialize, Serializer};
use serde_json::value::Value;

use std::marker::PhantomData;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use http::request::Request;
use hyper::body::{to_bytes, Body};
use std::convert::Into;
use tower::Service;
use url::Url;

use vk_method::{Method, PairsArray, Params};

/// One method processing unit based on [`Instance`]
pub struct Worker<C>
where
    C: Service<Request<Body>> + Send,
{
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: JoinHandle<()>,
    phantom: PhantomData<Instance<C>>,
}

impl<C> Worker<C>
where
    C: Service<Request<Body>, Response = http::Response<Body>> + Send + 'static,
    <C as Service<Request<Body>>>::Future: Send,
    <C as Service<Request<Body>>>::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(id: usize, instance: Instance<C>, receiver: TaskReceiver) -> Self {
        let thread = tokio::spawn(async {
            Self::thread_loop(instance, receiver);
        });

        Self {
            thread,
            id,
            phantom: PhantomData,
        }
    }

    async fn thread_loop(mut instance: Instance<C>, receiver: TaskReceiver) -> Option<()> {
        loop {
            let mut receiver = receiver.lock().await;

            let message = receiver.recv().await?;

            match message {
                Message::NewMethod(method, sender) => {
                    let mut methods_with_senders =
                        Self::take_methods(&mut receiver, (MAX_METHODS_IN_EXECUTE - 1) as usize)
                            .ok()?;

                    if methods_with_senders.is_empty() {
                        Self::process_method(&method, &sender, &mut instance);
                    } else {
                        // Self::process_execute()
                        todo!()
                    }
                }
            }

            // match receiver.recv().await {
            //     Some(message) => match message {
            //         Message::NewMethod(method, sender) => {
            //             let mut methods: Vec<Method> = Vec::new();
            //             let mut senders: Vec<ResultSender> = Vec::new();
            //
            //             //because first has already been received
            //             'method_collection: for _ in 0..MAX_METHODS_IN_EXECUTE - 1 {
            //                 match receiver.try_recv() {
            //                     Ok(message) => match message {
            //                         Message::NewMethod(method, sender) => {
            //                             methods.push(method);
            //                             senders.push(sender);
            //                         }
            //                         Message::Terminate => break 'thread_loop,
            //                     },
            //                     Err(reason) => match reason {
            //                         mpsc::error::TryRecvError::Empty => break 'method_collection,
            //                         mpsc::error::TryRecvError::Disconnected => break 'thread_loop,
            //                     },
            //                 }
            //             }
            //
            //             if methods.is_empty() {
            //                 let request = Self::prepare_request(&method, &mut instance);
            //                 let request_future = instance.http_client.call(request);
            //
            //                 tokio::spawn(async {
            //                     let value = Self::handle_request(request_future).await;
            //                     sender.send(value.map_err(Into::into)).unwrap();
            //                 });
            //             } else {
            //                 methods.push(method);
            //                 senders.push(sender);
            //
            //                 Self::handle_execute(methods, senders, &instance);
            //             };
            //         }
            //         Message::Terminate => break,
            //     },
            //     None => {
            //         break;
            //     }
            // }
            //
            //important! Unlock mutex before sleep
            drop(receiver);
            sleep(instance.time_between_requests).await;
        }
    }

    fn process_method(method: &Method, sender: &ResultSender, instance: &mut Instance<C>) {
        let request = Self::prepare_request(method, instance);
        let request_future = instance.http_client.call(request);

        tokio::spawn(async {
            let value = Self::handle_request(request_future).await;
            sender.send(value.map_err(Into::into)).unwrap();
        });
    }

    fn take_methods(
        receiver: &mut mpsc::UnboundedReceiver<Message>,
        count: usize,
    ) -> Result<Vec<(Method, ResultSender)>, mpsc::error::TryRecvError> {
        let mut methods: Vec<(Method, ResultSender)> = Vec::new();

        for i in 0..count {
            let message = receiver.try_recv();

            if let Err(mpsc::error::TryRecvError::Empty) = message {
                break;
            }

            match message? {
                Message::NewMethod(method, sender) => methods.push((method, sender)),
            }
        }

        Ok(methods)
    }

    fn prepare_request(method: &Method, instance: &mut Instance<C>) -> Request<Body> {
        let mut url = Url::parse(&format!("{}/method/{}", &instance.api_url, method.name)).unwrap();

        {
            let mut pairs = url.query_pairs_mut();
            query(&mut pairs, &[("access_token", &instance.token)]);
            query(&mut pairs, &[("v", &instance.api_version)]);
            query(&mut pairs, &method.params);
        }

        http::Request::post(url.to_string())
            .header("Content-Length", 0)
            .body(Body::empty())
            .unwrap()
    }

    async fn handle_request(
        request: <C as Service<Request<Body>>>::Future,
    ) -> anyhow::Result<Value> {
        let mut response = request.await?;
        let value: Value = serde_json::from_slice(&to_bytes(response.body_mut()).await?)?;

        Ok(value)
    }

    fn parse_execute(response: Value) -> Result<Vec<Result<Value, VkError>>, crate::Error> {
        let mut execute_errors: Vec<VkError> = serde_json::from_value(
            response
                .as_object_mut()
                .unwrap()
                .remove("execute_error")
                .unwrap_or_default(),
        )
        .unwrap();

        let execute_response: Result<Value, VkError> =
            serde_json::from_value::<VkResult<Value>>(response)
                .unwrap()
                .into();

        let responses: Vec<Value> =
            serde_json::from_value(execute_response?).map_err(anyhow::Error::new)?;

        let mut result = Vec::new();

        for response in responses {
            if let Value::Bool(false) = response {
                result.push(Err(execute_errors.remove(0)));
            } else {
                result.push(Ok(response));
            }
        }

        Ok(result)
    }

    async fn process_execute(
        methods_with_senders: Vec<(Method, ResultSender)>,
        instance: &mut Instance<C>,
    ) -> Result<Vec<Result<Value, VkError>>, crate::Error> {
        let (methods, senders): (Vec<_>, Vec<_>) = methods_with_senders.into_iter().unzip();
        let execute = ExecuteCompiler::compile(methods);

        let execute = Method::new(
            "execute",
            Params::try_from(PairsArray([("code", execute)])).unwrap(),
        );

        let request = Self::prepare_request(&execute, instance);

        // add tokio spawn
        let requst_future = instance.http_client.call(request);
        let mut response = Self::handle_request(requst_future).await?;
        
        let result = Self::parse_execute(response);
        //     let mut raw_response = match req.await {
        //         Ok(response) => match response.json().await {
        //             Ok(result) => result,
        //             Err(error) => {
        //                 let error = Arc::new(Error::Custom(error.into()));
        //
        //                 for sender in senders {
        //                     sender.send(Err(Error::Arc(Arc::clone(&error)))).unwrap();
        //                 }
        //
        //                 return;
        //             }
        //         },
        //         Err(error) => {
        //             let error = Arc::new(Error::Custom(error.into()));
        //
        //             for sender in senders {
        //                 sender.send(Err(Error::Arc(Arc::clone(&error)))).unwrap();
        //             }
        //
        //             return;
        //         }
        //     };
        //
        //     let execute_errors_raw = if let Value::Object(ref mut map) = raw_response {
        //         map.remove("execute_errors")
        //     } else {
        //         None
        //     };
        //
        //     let mut execute_errors: Vec<VkError> = Vec::new();
        //
        //     if let Some(execute_errors_value) = execute_errors_raw {
        //         execute_errors = serde_json::from_value(execute_errors_value).unwrap();
        //     }
        //
        //     let response: Result<Value, VkError> =
        //         serde_json::from_value::<VkResult<Value>>(raw_response)
        //         .unwrap()
        //         .into();
        //
        // match response {
        //     Ok(responses) => {
        //         let responses: Vec<Value> = serde_json::from_value(responses).unwrap();
        //
        //         for (sender, response) in senders.into_iter().zip(responses.into_iter()) {
        //             if let Some(bool) = response.as_bool() {
        //                 if bool == false {
        //                     sender
        //                         .send(Err(Error::VK(execute_errors.remove(0))))
        //                         .unwrap();
        //                 }
        //             } else {
        //                 sender.send(Ok(response)).unwrap();
        //             }
        //         }
        //     }
        //     Err(error) => {
        //         let error = Arc::new(Error::VK(error));
        //
        //             for sender in senders {
        //                 sender.send(Err(Error::Arc(Arc::clone(&error)))).unwrap();
        //             }
        //         }
        //     }
        // });
    }
}

fn query<'input, 'output, Target>(
    pairs: &'output mut url::form_urlencoded::Serializer<'input, Target>,
    query: &impl Serialize,
) where
    Target: 'output + url::form_urlencoded::Target,
{
    let serializer = comma_serde_urlencoded::Serializer::new(pairs);
    query.serialize(serializer).unwrap();
}
