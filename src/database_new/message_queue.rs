use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self};

pub(crate) mod errors {
    use std::convert::From;
    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::oneshot::error::RecvError;

    pub(crate) struct QueueDrop {}

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

use errors::QueueDrop;

pub(crate) struct Sender<M, R> {
    tx: UnboundedSender<(M, oneshot::Sender<R>)>,
}

impl<M, R> Clone for Sender<M, R> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<M, R> Sender<M, R> {
    pub(crate) async fn send_for_response(&self, msg: M) -> Result<R, QueueDrop> {
        let (tx, rx) = oneshot::channel();

        self.tx.send((msg, tx))?;
        let res = rx.await?;

        Ok(res)
    }

    pub(crate) fn send(&self, msg: M) -> Result<(), QueueDrop> {
        let (tx, _) = oneshot::channel();
        self.tx.send((msg, tx))?;

        Ok(())
    }
}

pub(crate) struct Queue<M, R> {
    rx: UnboundedReceiver<(M, oneshot::Sender<R>)>,
}

impl<M, R> Queue<M, R> {
    pub(crate) async fn recv(&mut self) -> Option<(M, oneshot::Sender<R>)> {
        self.rx.recv().await
    }
}

pub(super) fn create<M, R>() -> (Sender<M, R>, Queue<M, R>) {
    let (tx, rx) = unbounded_channel();
    (Sender { tx }, Queue { rx })
}
