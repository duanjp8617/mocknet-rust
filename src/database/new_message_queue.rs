use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot;
use tokio::sync::mpsc::error::TryRecvError;

pub mod error {
    use std::fmt;
    use std::convert::From;

    use tokio::sync::mpsc::error as mpsc;
    use tokio::sync::oneshot::error as oneshot;

    #[derive(Debug)]
    pub enum MsgQError {
        MpscSendError,
        OneshotRecvError,
    }


    impl<T> From<mpsc::SendError<T>> for MsgQError {
        fn from(_: mpsc::SendError<T>) -> MsgQError {
            MsgQError::MpscSendError
        }
    }

    impl From<oneshot::RecvError> for MsgQError {
        fn from(_: oneshot::RecvError) -> MsgQError {
            MsgQError::OneshotRecvError
        }
    }

    impl fmt::Display for MsgQError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                MsgQError::MpscSendError => {
                    write!(f, "message queue closed with mpsc::SendError")
                }
                MsgQError::OneshotRecvError => {
                    write!(f, "message queue closed with oneshot::RecvError")
                }
            }            
        }
    }

    impl std::error::Error for MsgQError {}
}

// A message with a callback channel
pub struct Message<M, R, E> {
    msg: Option<M>,
    cb_tx: oneshot::Sender<Result<R, E>>,
}

impl<M, R, E> Message<M, R, E> {
    pub fn is_close_msg(&self) -> bool {
        self.msg.is_none()
    }

    pub fn try_get_msg(&mut self) -> Option<M> {
        self.msg.take()
    }

    pub fn callback(self, response: Result<R, E>) -> Result<(), Result<R,E>> {
        self.cb_tx.send(response)
    }
}

impl<M, R, E> Message<M, R, E> {
    fn new(msg: M, cb_tx: oneshot::Sender<Result<R, E>>) -> Self {
        Self{
            msg: Some(msg), 
            cb_tx
        }
    }

    fn close_msg() -> Self {
        Self {
            msg: None,
            cb_tx: oneshot::channel().0,
        }
    }
}

pub struct Sender<M, R> {
    tx: UnboundedSender<Message<M, R>>,
}

impl<M, R> Sender<M, R> {
    pub async fn send(&self, msg: M) -> Result<R, error::MsgQError> {
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

    pub fn close(&mut self) {
        self.rx.close()
    }

    pub fn try_recv(&mut self) -> Result<Message<M, R>, TryRecvError> {
        self.rx.try_recv()
    }
}

pub fn create<M, R>() -> (Sender<M, R>, Queue<M, R>) {
    let (tx, rx) = unbounded_channel();
    (Sender{tx}, Queue{rx})
}



