use crossbeam_channel::unbounded;

use std::future::Future;

mod worker;
use worker::Worker;
use worker::Message;
// pub struct InstancePool {
//     sender: crossbeam_channel::Sender<Message>,
//     workers: Vec<Worker>,
// }

// impl InstancePool {
//     pub fn new(instances: Vec<Instance>) -> InstancePool {
//         let mut workers = Vec::with_capacity(instances.len());
//         let (sender, receiver) = unbounded();

//         for (index, instance) in instances.into_iter().enumerate() {
//             workers.push(Worker::new(index, instance, receiver.clone()));
//         }

//         InstancePool { workers, sender }
//     }

//     pub fn run<F>(&self, f: F) -> Result<(), crossbeam_channel::SendError<Message>>
//     where
//         F: Future<Output = ()> + Send + 'static,
//     {

//         self.sender.send(Message::NewTask())
//     }
// }

