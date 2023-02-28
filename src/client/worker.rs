use crate::{Result, VkError, VkResult};
use std::result::Result as StdResult;

use super::{Config, HttpsClient, Message, ResultSender, TaskReceiver, MAX_METHODS_IN_EXECUTE};

use tokio::sync::mpsc;
use vk_execute_compiler::ExecuteCompiler;

use serde::Serialize;
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

/// One method processing unit based on [`Config`]
pub struct Worker<C: HttpsClient>
where
    <C as Service<Request<Body>>>::Future: Send,
{
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: JoinHandle<()>,
    phantom: PhantomData<Config<C>>,
}

impl<C: HttpsClient> Worker<C>
where
    <C as Service<Request<Body>>>::Future: Send,
{
    pub fn new(id: usize, config: Config<C>, receiver: TaskReceiver) -> Self {
        let thread = tokio::spawn(async {
            Self::thread_loop(config, receiver).await;
        });

        Self {
            thread,
            id,
            phantom: PhantomData,
        }
    }

    async fn thread_loop(mut config: Config<C>, receiver: TaskReceiver) -> Option<()> {
        loop {
            let mut receiver = receiver.lock().await;

            let message = receiver.recv().await?;

            match message {
                Message::NewMethod(method, sender) => {
                    let mut methods_with_senders =
                        Self::take_methods(&mut receiver, (MAX_METHODS_IN_EXECUTE - 1) as usize)
                            .ok()?;

                    if methods_with_senders.is_empty() {
                        Self::process_method(&method, sender, &mut config);
                    } else {
                        methods_with_senders.push((method, sender));
                        Self::process_execute(methods_with_senders, &mut config);
                    }
                }
            }

            //important! Unlock mutex before sleep
            drop(receiver);
            sleep(config.time_between_requests).await;
        }
    }

    /// Complete single method process up to sending result
    fn process_method(method: &Method, sender: ResultSender, config: &mut Config<C>) {
        let request = Self::prepare_request(method, config);
        let request_future = config.http_client.call(request);

        tokio::spawn(async {
            let result = Self::handle_method(request_future).await;
            sender.send(result).unwrap();
        });
    }

    async fn handle_method(request_future: <C as Service<Request<Body>>>::Future) -> Result<Value> {
        let response = Self::handle_request(request_future).await?;

        let result = <StdResult<Value, VkError>>::from(
            serde_json::from_value::<VkResult<Value>>(response).unwrap(),
        );

        result.map_err(Into::into)
    }

    /// Takes methods from receiver until it becomes empty or reach `max`
    fn take_methods(
        receiver: &mut mpsc::UnboundedReceiver<Message>,
        max: usize,
    ) -> StdResult<Vec<(Method, ResultSender)>, mpsc::error::TryRecvError> {
        let mut methods: Vec<(Method, ResultSender)> = Vec::new();

        for _ in 0..max {
            let message = receiver.try_recv();

            if matches!(message, Err(mpsc::error::TryRecvError::Empty)) {
                break;
            }

            match message? {
                Message::NewMethod(method, sender) => methods.push((method, sender)),
            }
        }

        Ok(methods)
    }

    fn prepare_request(method: &Method, config: &mut Config<C>) -> Request<Body> {
        let mut url = Url::parse(&format!("{}/method/{}", &config.api_url, method.name)).unwrap();

        {
            let mut pairs = url.query_pairs_mut();
            query(&mut pairs, &[("access_token", &config.token)]);
            query(&mut pairs, &[("v", &config.api_version)]);
            query(&mut pairs, &method.params);
        }

        http::Request::post(url.to_string())
            .header("Content-Length", 0)
            .body(Body::empty())
            .unwrap()
    }

    /// Makes request and tries to parse response to `Value`
    ///
    /// Note that this function don't handle parsed request in any way.
    /// It just parses a json
    async fn handle_request(
        request_future: <C as Service<Request<Body>>>::Future,
    ) -> Result<Value> {
        let mut response = request_future.await.map_err(Arc::new)?;
        let value: Value =
            serde_json::from_slice(&to_bytes(response.body_mut()).await.map_err(Arc::new)?).unwrap();

        Ok(value)
    }

    /// Parses execute from `serde_json::Value` to `Result<Vec<StdResult<Value, crate::VkError>>>`
    /// where
    ///     outer the result stands for possible shared error
    ///     the inner result stands for possible owned vk error
    fn parse_execute(mut response: Value) -> Result<Vec<StdResult<Value, crate::VkError>>> {
        let mut execute_errors: Vec<VkError> = response
            .as_object_mut()
            .unwrap()
            .remove("execute_error")
            .map_or_else(Vec::new, |errors| serde_json::from_value(errors).unwrap());

        let execute_response = <StdResult<Value, VkError>>::from(
            serde_json::from_value::<VkResult<Value>>(response).unwrap(),
        )
        .map_err(Arc::new)?;

        let responses: Vec<Value> = serde_json::from_value(execute_response).unwrap();

        let mut result = Vec::new();

        for response in responses {
            if response == Value::Bool(false) {
                result.push(Err(execute_errors.remove(0)));
            } else {
                result.push(Ok(response));
            }
        }

        Ok(result)
    }

    fn send_execute_results(
        result: Result<Vec<StdResult<Value, crate::VkError>>>,
        senders: Vec<ResultSender>,
    ) {
        if let Err(error) = result {
            for sender in senders {
                sender.send(Err(error.clone())).unwrap();
            }
            return;
        };

        for (sender, result) in senders.into_iter().zip(result.unwrap()) {
            sender.send(result.map_err(Into::into)).unwrap();
        }
    }

    /// Complete `execute` method process up to sending results
    fn process_execute(methods_with_senders: Vec<(Method, ResultSender)>, config: &mut Config<C>) {
        let (methods, senders): (Vec<_>, Vec<_>) = methods_with_senders.into_iter().unzip();
        let execute = ExecuteCompiler::compile(methods);

        let execute = Method::new(
            "execute",
            Params::try_from(PairsArray([("code", execute)])).unwrap(),
        );

        let request = Self::prepare_request(&execute, config);

        let request_future = config.http_client.call(request);

        tokio::spawn(async {
            let result = Self::handle_execute(request_future).await;
            Self::send_execute_results(result, senders);
        });
    }

    /// Makes request and parses a response
    async fn handle_execute(
        request_future: <C as Service<Request<Body>>>::Future,
    ) -> Result<Vec<StdResult<Value, crate::VkError>>> {
        let response = Self::handle_request(request_future).await?;

        Self::parse_execute(response)
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
