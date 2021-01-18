use std::convert::From;

use tokio::sync::mpsc::error::{SendError, TryRecvError};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, error::RecvError};

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

    pub fn callback(self, response: Result<R, E>) -> Result<(), Result<R, E>> {
        self.cb_tx.send(response)
    }
}

impl<M, R, E> Message<M, R, E> {
    fn new(msg: M, cb_tx: oneshot::Sender<Result<R, E>>) -> Self {
        Self {
            msg: Some(msg),
            cb_tx,
        }
    }

    // fn close_msg() -> Self {
    //     Self {
    //         msg: None,
    //         cb_tx: oneshot::channel().0,
    //     }
    // }
}

pub struct Sender<M, R, E> {
    tx: UnboundedSender<Message<M, R, E>>,
}

impl<M, R, E> Clone for Sender<M, R, E> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<M, R, E> Sender<M, R, E>
where
    E: From<SendError<Message<M, R, E>>> + From<RecvError>,
{
    pub async fn send(&self, msg: M) -> Result<R, E> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::new(msg, tx);

        self.tx.send(msg)?;
        let res = rx.await?;

        res
    }
}

pub struct Queue<M, R, E> {
    rx: UnboundedReceiver<Message<M, R, E>>,
}

impl<M, R, E> Queue<M, R, E> {
    pub async fn recv(&mut self) -> Option<Message<M, R, E>> {
        self.rx.recv().await
    }

    pub fn close(&mut self) {
        self.rx.close()
    }

    pub fn try_recv(&mut self) -> Result<Message<M, R, E>, TryRecvError> {
        self.rx.try_recv()
    }
}

pub fn create<M, R, E>() -> (Sender<M, R, E>, Queue<M, R, E>) {
    let (tx, rx) = unbounded_channel();
    (Sender { tx }, Queue { rx })
}
