use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot;

pub mod error {
    use std::fmt;
    use std::convert::From;

    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::oneshot::error::RecvError;

    #[derive(Debug)]
    pub struct SenderError {
        description: String,
    }

    impl<T> From<SendError<T>> for SenderError {
        fn from(err: SendError<T>) -> Self {
            Self {description: format!("SendError from tokio's mpsc channel: {}", err)}
        }
    }

    impl From<RecvError> for SenderError {
        fn from(err: RecvError) -> Self {
            Self {description: format!("RecvError from tokio's oneshot channel: {}", err)}
        }
    }

    impl fmt::Display for SenderError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", &self.description)
        }
    }

    impl std::error::Error for SenderError {
        fn description(&self) -> &str {
            &self.description
        }
    }
}

// A message with a callback channel
pub struct Message<M, R> {
    msg: M,
    cb_tx: oneshot::Sender<R>,
}

impl<M, R> Message<M, R> {
    pub fn take_inner(self) -> (M, oneshot::Sender<R>) {
        let msg = self.msg;
        let cb_tx = self.cb_tx;
        (msg, cb_tx)
    }

    fn new(msg: M, cb_tx: oneshot::Sender<R>) -> Self {
        Self{msg, cb_tx}
    }
}

pub struct Sender<M, R> {
    tx: UnboundedSender<Message<M, R>>,
}

impl<M, R> Sender<M, R> {
    pub async fn send(&self, msg: M) -> Result<R, error::SenderError> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::new(msg, tx);
        self.tx.send(msg)?;
        Ok(rx.await?)
    }
}

pub struct Queue<M, R> {
    rx: UnboundedReceiver<Message<M, R>>,
}

impl<M, R> Queue<M, R> {
    pub async fn recv(&mut self) -> Option<Message<M, R>>  {
        self.rx.recv().await
    }
}

pub fn create<M, R>() -> (Sender<M, R>, Queue<M, R>) {
    let (tx, rx) = unbounded_channel();
    (Sender{tx}, Queue{rx})
}



