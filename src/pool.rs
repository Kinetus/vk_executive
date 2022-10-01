pub mod instance;
mod event;
mod message;
mod execute_manager;
mod worker;
mod task_observer;

pub use instance::Instance;
pub use event::Event;
use message::Message;
use execute_manager::ExecuteManager;
use worker::Worker;
use task_observer::TaskObserver;

pub type Sender = oneshot::Sender<Result<Value>>;
pub type TaskSender = mpsc::UnboundedSender<Message>;
pub type TaskReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Message>>>;
pub type EventSender = broadcast::Sender<Event>;
pub type EventReceiver = broadcast::Receiver<Event>;

use vk_method::Method;
use crate::Result;

use serde_json::value::Value;

use std::iter::ExactSizeIterator;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

pub struct InstancePool {
    sender: TaskSender,
    workers: Vec<Worker>,
    execute_manager: ExecuteManager,
    task_observer: TaskObserver,
    event_sender: EventSender,
}

impl InstancePool {
    pub fn from_instances<Instances>(instances: Instances) -> Self
    where
        Instances: IntoIterator<Item = Instance>,
        <Instances as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let instances = instances.into_iter();
        let mut workers = Vec::with_capacity(instances.len());

        let (sender, receiver) = mpsc::unbounded_channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let (event_sender, event_receiver) = broadcast::channel(20000);

        for (index, instance) in instances.into_iter().enumerate() {
            workers.push(Worker::new(
                index,
                instance,
                receiver.clone(),
                event_sender.clone(),
            ));
        }

        InstancePool {
            workers,
            execute_manager: ExecuteManager::new(event_sender.subscribe(), sender.clone()),
            task_observer: TaskObserver::new(event_receiver),
            sender,
            event_sender
        }
    }

    pub async fn run(&self, method: Method) -> Result<Value> {
        self.event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(2)).await; //tokio broadcast so slow and need sleep

        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        let running_tasks = self.task_observer.running_task().await;

        if running_tasks < self.workers.len() {
            self.sender
                .send(Message::NewMethod(method, oneshot_sender))
                .unwrap();
        } else {
            self.execute_manager.push(method, oneshot_sender)?;
        };

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
