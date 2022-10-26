mod message;
mod worker;

use crate::Instance;
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

pub const MAX_METHODS_IN_EXECUTE: u8 = 25;

/// An asynchronous `Client` to make VK Requests with.
pub struct Client {
    sender: TaskSender,
    workers: Vec<Worker>,
}

impl Client {
    /// Builds `Client` from any type that can be converted into ExactSizeIterator over Instance
    pub fn from_instances<Instances>(instances: Instances) -> Self
    where
        Instances: IntoIterator<Item = Instance>,
        <Instances as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let instances = instances.into_iter();
        let mut workers = Vec::with_capacity(instances.len());

        let (sender, receiver) = mpsc::unbounded_channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for (index, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(index, instance, receiver.clone()));
        }

        Client { workers, sender }
    }

    /// Asynchronously sends [`Method`]
    ///
    /// # Example:
    ///
    /// ```rust
    /// use fast_vk::{Instance, Client};
    /// use vk_method::{Method, Params};
    /// #
    /// # use std::env;
    /// # use dotenv::dotenv;
    /// # dotenv().unwrap();
    /// #
    /// # async fn main() {
    /// # let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(1)).unwrap();
    /// # let pool = Client::from_instances(instances);
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
    pub async fn method(&self, method: Method) -> Result<Value> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        self.sender
            .send(Message::NewMethod(method, oneshot_sender))
            .unwrap();

        oneshot_receiver.await.unwrap()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
    }
}

#[cfg(feature = "thisvk")]
#[async_trait::async_trait]
impl thisvk::API for Client {
    type Error = crate::Error;

    async fn method<T>(&self, method: Method) -> Result<T>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        serde_json::from_value(self.method(method).await?)
            .map_err(|error| crate::Error::Custom(error.into()))
    }
}
