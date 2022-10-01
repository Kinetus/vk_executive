use super::{Event, EventReceiver};

use std::panic;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub struct TaskObserver {
    running_tasks: Arc<RwLock<usize>>,
    thread: Option<JoinHandle<()>>,
}

impl TaskObserver {
    pub fn new(mut receiver: EventReceiver) -> TaskObserver {
        let running_tasks = Arc::new(RwLock::new(0));

        let running_tasks_inner = Arc::clone(&running_tasks);
        let thread = Some(tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => match event {
                        Event::GotWork => {
                            *running_tasks_inner.write().await += 1;
                        }
                        Event::DoneWork => {
                            *running_tasks_inner.write().await -= 1;
                        }
                    },
                    Err(_) => break,
                }
            }
        }));

        TaskObserver {
            thread,
            running_tasks,
        }
    }

    pub async fn running_task(&self) -> usize {
        // we can use unwrap safe because only drop function takes thread
        if self.thread.as_ref().unwrap().is_finished() {
            drop(&self);
        }

        *self.running_tasks.read().await
    }
}

impl Drop for TaskObserver {
    fn drop(&mut self) {
        let thread = self.thread.take().unwrap();

        if thread.is_finished() {
            let result = futures::executor::block_on(async { thread.await });

            if let Err(err) = result {
                if let Ok(reason) = err.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        } else {
            thread.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn got_work_two_times() {
        let (event_sender, event_receiver) = broadcast::channel(2);

        let worker_watcher = TaskObserver::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        assert_eq!(worker_watcher.running_task().await, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn one_of_two_done_work() {
        let (event_sender, event_receiver) = broadcast::channel(2);

        let worker_watcher = TaskObserver::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        event_sender.send(Event::DoneWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        assert_eq!(worker_watcher.running_task().await, 1);
    }

    #[tokio::test]
    async fn two_done_work() {
        let (event_sender, event_receiver) = broadcast::channel(2);

        let worker_watcher = TaskObserver::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        event_sender.send(Event::DoneWork).unwrap();
        event_sender.send(Event::DoneWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        assert_eq!(worker_watcher.running_task().await, 0);
    }

    #[tokio::test]
    #[should_panic]
    async fn done_more_than_got() {
        let (event_sender, event_receiver) = broadcast::channel(3);

        let _worker_watcher = TaskObserver::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        event_sender.send(Event::DoneWork).unwrap();
        event_sender.send(Event::DoneWork).unwrap();
        event_sender.send(Event::DoneWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;
    }
}
