pub mod instance;
mod message;
mod worker;

pub use instance::Instance;
use message::Message;
use worker::Worker;

pub type Sender = oneshot::Sender<Result<Value>>;
pub type TaskSender = mpsc::UnboundedSender<Message>;
pub type TaskReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Message>>>;

use vk_method::Method;
use crate::Result;

use serde_json::value::Value;

use std::iter::ExactSizeIterator;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};
use core::future::Future;

pub const MAX_METHODS_IN_EXECUTE: u8 = 25;

/// An asynchronous `Client` to make Requests with.
pub struct Client {
    sender: TaskSender,
    workers: Vec<Worker>,
}

impl Client {
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
            workers.push(Worker::new(
                index,
                instance,
                receiver.clone()
            ));
        }

        Client {
            workers,
            sender
        }
    }

    pub fn run(&self, method: Method) -> impl Future<Output = Result<Value>> + '_ {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        async move {
            self.sender
                .send(Message::NewMethod(method, oneshot_sender))
                .unwrap();

            oneshot_receiver.await.unwrap()
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        drop(&self.sender);
    }
}

#[cfg(feature = "thisvk")]
#[async_trait::async_trait]
impl thisvk::API for Client {
    type Error = crate::Error;

    async fn method<T>(&self, method: Method) -> Result<T>
    where for<'de>
        T: serde::Deserialize<'de>
    {
        match self.run(method).await {
            Ok(value) => match serde_json::from_value(value) {
                Ok(result) => Ok(result),
                Err(error) => Err(crate::Error::Custom(error.into()))
            },
            Err(error) => Err(error)
        }
    }
}