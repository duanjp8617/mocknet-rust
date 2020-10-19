use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot;

pub mod error {
    use std::fmt;
    use std::convert::From;

    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::oneshot::error::RecvError;

    #[derive(Debug)]
    pub struct Error {
        description: String,
    }

    impl<T> From<SendError<T>> for Error {
        fn from(err: SendError<T>) -> Self {
            Self {description: format!("SenderError: {}", err)}
        }
    }

    impl From<RecvError> for Error {
        fn from(err: RecvError) -> Self {
            Self {description: format!("ResponseRecvError: {}", err)}
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", &self.description)
        }
    }

    impl std::error::Error for Error {
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
    pub fn msg(&self) -> &M {
        &self.msg
    }

    pub fn msg_mut(&mut self) -> &mut M {
        &mut self.msg
    }

    pub fn callback(self, response: R) -> Result<(), R> {
        let cb_tx = self.take_cb_tx();
        cb_tx.send(response)
    }

    fn take_cb_tx(self) -> oneshot::Sender<R> {
        let cb_tx = self.cb_tx;
        cb_tx
    }

    fn new(msg: M, cb_tx: oneshot::Sender<R>) -> Self {
        Self{msg, cb_tx}
    }
}

pub struct Sender<M, R> {
    tx: UnboundedSender<Message<M, R>>,
}

impl<M, R> Sender<M, R> {
    pub async fn send(&self, msg: M) -> Result<R, error::Error> {
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



