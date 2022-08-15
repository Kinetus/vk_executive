use super::Message;
use super::{Method, MethodWithSender};

use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

mod event;
pub use event::Event;

pub struct ExecuteManager {
    queue: Arc<Mutex<Vec<MethodWithSender>>>,
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

    fn push_execute(queue: &mut Vec<MethodWithSender>, work_sender: &crossbeam_channel::Sender<Message>) {
        if queue.len() > 0 {
            let methods_count = if queue.len() < 24 { queue.len() } else { 24 };
            let methods_with_senders = queue.drain(0..methods_count);

            let mut methods = Vec::new();
            let mut senders = Vec::new();

            for MethodWithSender { method, sender } in methods_with_senders {
                methods.push(method);
                senders.push(sender);
            }

            work_sender
                .send(Message::NewExecute(methods, senders))
                .unwrap();
        }
    }

    pub fn push(&self, method: MethodWithSender) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(method);
    }

    pub fn compile_execute(execute: Vec<Method>) -> String {
        let mut code = String::new();

        let method_count = execute.len();

        for (index, method) in execute.into_iter().enumerate() {
            code.push_str(
                format!(
                    "var result{index} = API.{}({});",
                    method.name,
                    serde_json::to_string(&method.params).unwrap()
                )
                .as_str(),
            );
        }

        code.push_str("return [");

        for i in 0..method_count {
            code.push_str(format!("result{i},").as_str());
        }

        code.push_str("];");

        code
    }
}
