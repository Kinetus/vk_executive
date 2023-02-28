mod message;
mod worker;

use crate::Config;
use message::Message;
use worker::Worker;

pub type ResultSender = oneshot::Sender<Result<Value>>;
pub type TaskSender = mpsc::UnboundedSender<Message>;
pub type TaskReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Message>>>;

use crate::Result;
use vk_method::Method;

use serde_json::value::Value;

use std::iter::ExactSizeIterator;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};

use http::request::Request;
use hyper::body::Body;
use tower::Service;

pub const MAX_METHODS_IN_EXECUTE: u8 = 25;

// https://github.com/rust-lang/rust/issues/41517#issuecomment-1100644808
pub trait HttpsClient:
    Service<Request<Body>, Response = http::Response<Body>, Error = hyper::Error>
    + Send
    + Sync
    + 'static
where
    Self::Future: Send,
{
}

impl<T> HttpsClient for T
where
    T: Service<Request<Body>, Response = http::Response<Body>, Error = hyper::Error>
        + Send
        + Sync
        + 'static,
    Self::Future: Send,
{
}

pub type HyperClient = hyper::client::Client<HttpsConnector<HttpConnector>>;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

/// An asynchronous `Client` to make VK Requests with.
pub struct Client<C: HttpsClient>
where
    <C as Service<Request<Body>>>::Future: Send,
{
    sender: TaskSender,
    #[allow(dead_code)]
    workers: Vec<Worker<C>>,
}

impl<C: HttpsClient> Client<C>
where
    <C as Service<Request<Body>>>::Future: Send,
{
    /// Builds `Client` from any `ExactSizeIterator` over Config
    pub fn from_configs<Configs>(configs: Configs) -> Self
    where
        Configs: Iterator<Item = Config<C>> + ExactSizeIterator,
    {
        let mut workers = Vec::with_capacity(configs.len());

        let (sender, receiver) = mpsc::unbounded_channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for (index, config) in configs.into_iter().enumerate() {
            workers.push(Worker::new(index, config, receiver.clone()));
        }

        Self { sender, workers }
    }

    /// Asynchronously sends [`Method`]
    ///
    /// # Example:
    ///
    /// ```rust
    /// use vk_executive::{Config, Client};
    /// use vk_method::{Method, Params};
    /// #
    /// # use std::env;
    /// # use dotenv::dotenv;
    /// # dotenv().unwrap();
    /// #
    /// # async fn main() {
    /// # let configs = Config::from_tokens(env::var("tokens").unwrap().split(",").take(1)).unwrap();
    /// # let pool = Client::from_configs(configs.into_iter());
    ///
    /// let mut params = Params::new();
    /// params.insert("user_id", 1);
    ///
    /// let response = pool.method(Method::new(
    ///     "users.get",
    ///     params
    /// )).await.unwrap();
    ///
    /// assert_eq!(
    ///     response,
    ///     serde_json::json!([
    ///         {
    ///             "id": 1,
    ///             "first_name": "Pavel",
    ///             "last_name": "Durov",
    ///             "is_closed": false,
    ///             "can_access_closed": true
    ///         }
    ///     ])
    /// );
    /// # }
    /// ```
    /// # Errors
    /// If this function encounters any form of network, serialization or VK error, an error variant will be returned.
    ///
    /// # Panics
    ///
    /// If the method name starts with `execute`, the function will panic.
    /// `Client` itself creates execute requests, so you don't need to use it explicitly.
    pub async fn method(&self, method: Method) -> Result<Value> {
        assert!(
            !method.name.starts_with("execute"),
            "Execute method is not allowed"
        );
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        self.sender
            .send(Message::NewMethod(method, oneshot_sender))
            .unwrap();

        oneshot_receiver.await.unwrap()
    }
}

#[cfg(feature = "thisvk")]
#[async_trait::async_trait]
impl<C: HttpsClient> thisvk::API for Client<C>
where
    <C as Service<Request<Body>>>::Future: Send,
{
    type Error = crate::Error;

    async fn method<T>(&self, method: Method) -> Result<T>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        let value = self.method(method).await?;
        Ok(serde_json::from_value(value).unwrap())
    }
}
