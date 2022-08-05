use tokio::sync::oneshot;
use super::method::Method;

pub enum Message {
    NewTask {
        method: Method,
        oneshot_sender: oneshot::Sender<Result<reqwest::Response, reqwest::Error>>,
    },
    Terminate,
}