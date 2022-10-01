use tokio::task::JoinHandle;
use super::EventReceiver;
use super::Event;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct WorkerWatcher {
    running_workers: Arc<RwLock<usize>>,
    thread: JoinHandle<()>
}

impl WorkerWatcher {
    pub fn new(mut receiver: EventReceiver) -> WorkerWatcher {
        let running_workers = Arc::new(RwLock::new(0));
        
        let running_workers_inner = Arc::clone(&running_workers);
        let thread = tokio::spawn(async move {
                loop {
                    match receiver.recv().await {
                        Ok(event) => match event {
                            Event::GotWork => {
                                *running_workers_inner.write().await += 1;
                            }
                            Event::DoneWork => {
                                *running_workers_inner.write().await -= 1;
                            }
                        },
                        Err(_) => break,
                    }
                }
            }
        );

        WorkerWatcher { thread, running_workers }
    }

    pub async fn finish(self) -> std::result::Result<(), tokio::task::JoinError> {
        self.thread.abort();

        match self.thread.await {
            Ok(_) => Ok(()),
            Err(err) => {
                if err.is_cancelled() {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
    
    pub async fn running_workers(&self) -> usize {
        *self.running_workers.read().await
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;
    use std::panic;
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn got_work_two_times() {
        let (event_sender, event_receiver) = broadcast::channel(2);

        let worker_watcher = WorkerWatcher::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        assert_eq!(worker_watcher.running_workers().await, 2);

        match worker_watcher.finish().await {
            Ok(_) => {},
            Err(err) => {
                if let Ok(reason) = err.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn one_of_two_done_work() {
        let (event_sender, event_receiver) = broadcast::channel(2);

        let worker_watcher = WorkerWatcher::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        event_sender.send(Event::DoneWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        assert_eq!(worker_watcher.running_workers().await, 1);

        match worker_watcher.finish().await {
            Ok(_) => {},
            Err(err) => {
                if let Ok(reason) = err.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        }
    }

    #[tokio::test]
    async fn two_done_work() {
        let (event_sender, event_receiver) = broadcast::channel(2);

        let worker_watcher = WorkerWatcher::new(event_receiver);

        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        event_sender.send(Event::DoneWork).unwrap();
        event_sender.send(Event::DoneWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        assert_eq!(worker_watcher.running_workers().await, 0);

        match worker_watcher.finish().await {
            Ok(_) => {},
            Err(err) => {
                if let Ok(reason) = err.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn done_more_than_got() {
        let (event_sender, event_receiver) = broadcast::channel(3);

        let worker_watcher = WorkerWatcher::new(event_receiver);
        
        event_sender.send(Event::GotWork).unwrap();
        event_sender.send(Event::GotWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        event_sender.send(Event::DoneWork).unwrap();
        event_sender.send(Event::DoneWork).unwrap();
        event_sender.send(Event::DoneWork).unwrap();
        tokio::time::sleep(std::time::Duration::from_nanos(1)).await;

        match worker_watcher.finish().await {
            Ok(_) => {},
            Err(err) => {
                if let Ok(reason) = err.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        }
    }
}