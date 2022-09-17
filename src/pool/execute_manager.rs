use super::Message;
use super::Sender;

use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

mod error;
pub use error::ExecuteError;

mod event;
pub use event::Event;
use vk_method::Method;

pub struct ExecuteManager {
    queue: Arc<Mutex<Vec<(Method, Sender)>>>,
    #[allow(dead_code)]
    sender: crossbeam_channel::Sender<Message>,
    #[allow(dead_code)]
    thread: JoinHandle<()>,
}

impl ExecuteManager {
    pub fn new(
        event_receiver: crossbeam_channel::Receiver<Event>,
        work_sender: crossbeam_channel::Sender<Message>,
    ) -> ExecuteManager {
        let queue = Arc::new(Mutex::new(Vec::new()));

        let thread_queue = Arc::clone(&queue);
        let sender = work_sender.clone();

        let thread = tokio::spawn(async move {
            loop {
                match event_receiver.recv() {
                    Ok(event) => match event {
                        #[allow(unused_must_use)]
                        Event::FreeWorker => {
                            ExecuteManager::push_execute(&mut thread_queue.lock().unwrap(), &work_sender);
                        }
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        ExecuteManager {
            thread,
            queue,
            sender,
        }
    }

    fn push_execute(queue: &mut Vec<(Method, Sender)>, work_sender: &crossbeam_channel::Sender<Message>) -> Result<(), anyhow::Error> {
        if queue.len() == 0 {
            return Err(ExecuteError::EmptyQueue.into())
        }

        let methods_len = if queue.len() < 25 { queue.len() } else { 25 };
        let methods_with_senders = queue.drain(0..methods_len);

        let mut methods = Vec::new();
        let mut senders = Vec::new();

        for (method, sender) in methods_with_senders {
            methods.push(method);
            senders.push(sender);
        }

        work_sender
            .send(Message::NewExecute(methods, senders))?;
        
        Ok(())
    }

    pub fn push(&self, method: Method, sender: Sender) -> Result<(), anyhow::Error> {
        let mut queue = self.queue.lock().unwrap();
        queue.push((method, sender));
        
        if queue.len() >= 25 {
            ExecuteManager::push_execute(&mut queue, &self.sender)?;
        }

        Ok(())
    }
}
