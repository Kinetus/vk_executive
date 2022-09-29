mod execute_manager;
mod instance;
mod message;
mod worker;

pub use instance::Instance;
use vk_method::Method;
use message::Message;

use tokio::sync::{mpsc, oneshot, Mutex};
use std::sync::Arc;

use std::iter::ExactSizeIterator;

use crate::Result;
use serde_json::value::Value;

use execute_manager::{Event, ExecuteManager};

use worker::Worker;

pub type Sender = oneshot::Sender<Result<Value>>;

pub type TaskSender = mpsc::UnboundedSender<Message>;
pub type TaskReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Message>>>;

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type EventReceiver = mpsc::UnboundedReceiver<Event>;

pub struct InstancePool {
    sender: TaskSender,
    receiver: TaskReceiver,
    workers: Vec<Worker>,
    execute_manager: ExecuteManager,
}

impl<Instances> From<Instances> for InstancePool
where 
    Instances: IntoIterator<Item = Instance>,
    <Instances as IntoIterator>::IntoIter: ExactSizeIterator
{
    fn from(instances: Instances) -> Self {
        let instances = instances.into_iter();

        let mut workers = Vec::with_capacity(instances.len());
        let (sender, receiver) = mpsc::unbounded_channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

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
            receiver
        }  
    }
}

impl InstancePool {
    pub async fn run(&self, method: Method) -> Result<Value> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        // match self.receiver.lock().await.poll_recv() {
        //     core::task::Poll::Pending => {
                self.sender
                .send(Message::NewMethod(
                    method,
                    oneshot_sender,
                ))
                .unwrap();
            // },
            // _ => {
            //     self.execute_manager.push(method, oneshot_sender)?;
            // }
        // };
        
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
