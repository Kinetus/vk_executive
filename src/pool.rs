mod execute_manager;
mod instance;
mod message;
mod worker;

pub use instance::Instance;
use vk_method::Method;
use message::Message;

use crossbeam_channel::unbounded;
use tokio::sync::oneshot;

use std::iter::ExactSizeIterator;

use crate::Result;
use serde_json::value::Value;

use execute_manager::ExecuteManager;

use worker::Worker;

pub type Sender = oneshot::Sender<Result<Value>>;

pub struct InstancePool {
    sender: crossbeam_channel::Sender<Message>,
    workers: Vec<Worker>,
    execute_manager: ExecuteManager,
}

impl InstancePool {
    pub fn new<Instances>(instances: Instances) -> InstancePool
    where
        Instances: ExactSizeIterator<Item = Instance>
    {
        let mut workers = Vec::with_capacity(instances.len());
        let (sender, receiver) = unbounded();

        let (event_sender, event_receiver) = unbounded();

        for (index, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(
                index,
                instance,
                receiver.clone(),
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

    pub async fn run(&self, method: Method) -> Result<Value> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        if self.sender.is_empty() {
            // ! 1 unnecessary method. Need fix
            self.sender
                .send(Message::NewMethod(
                    method,
                    oneshot_sender,
                ))
                .unwrap();
        } else {
            self.execute_manager.push(method, oneshot_sender)?;
        }

        oneshot_receiver.await.unwrap()
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
