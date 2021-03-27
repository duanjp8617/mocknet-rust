use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self};

mod errors {
    use std::convert::From;
    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::oneshot::error::RecvError;

    pub struct QueueDrop {}

    impl<T> From<SendError<T>> for QueueDrop {
        fn from(_: SendError<T>) -> QueueDrop {
            Self {}
        }
    }

    // Convert oneshot::RecvError into BackendError.
    impl From<RecvError> for QueueDrop {
        fn from(_: RecvError) -> QueueDrop {
            Self {}
        }
    }
}

pub use errors::QueueDrop;

// A message with a callback channel
pub struct Message<M, R> {
    msg: Option<M>,
    cb_tx: oneshot::Sender<R>,
}

impl<M, R> Message<M, R> {
    pub fn is_close_msg(&self) -> bool {
        self.msg.is_none()
    }

    pub fn try_get_msg(&mut self) -> Option<M> {
        self.msg.take()
    }

    pub fn callback(self, response: R) -> Result<(), R> {
        self.cb_tx.send(response)
    }
}

impl<M, R> Message<M, R> {
    fn new(msg: M, cb_tx: oneshot::Sender<R>) -> Self {
        Self {
            msg: Some(msg),
            cb_tx,
        }
    }
}

pub struct Sender<M, R> {
    tx: UnboundedSender<Message<M, R>>,
}

impl<M, R> Clone for Sender<M, R> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<M, R> Sender<M, R> {
    pub async fn send_for_response(&self, msg: M) -> Result<R, QueueDrop> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::new(msg, tx);

        self.tx.send(msg)?;
        let res = rx.await?;

        Ok(res)
    }

    pub fn send(&self, msg: M) -> Result<(), QueueDrop> {
        let (tx, _) = oneshot::channel();
        let msg = Message::new(msg, tx);

        self.tx.send(msg)?;

        Ok(())
    }
}

pub struct Queue<M, R> {
    rx: UnboundedReceiver<Message<M, R>>,
}

impl<M, R> Queue<M, R> {
    pub async fn recv(&mut self) -> Option<Message<M, R>> {
        self.rx.recv().await
    }

    pub fn close(&mut self) {
        self.rx.close()
    }
}

pub fn create<M, R>() -> (Sender<M, R>, Queue<M, R>) {
    let (tx, rx) = unbounded_channel();
    (Sender { tx }, Queue { rx })
}
