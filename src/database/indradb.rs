// An implementation of Indradb storage backend
use std::future::Future;
use std::task::{Context, Poll, Poll::Ready, Poll::Pending};
use std::pin::Pin;
use std::marker::Unpin;

use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::errors::Error;

// A request sent from the database client
pub(super) enum Request {
    Ping,
}

pub(super) enum Response {
    Ping(bool),
}

pub(super) struct IndradbConnLoop {
    rpc_system_driver: Pin<Box<dyn Future<Output = Result<(), Error>> + 'static>>,
    rpc_client_driver: Pin<Box<dyn Future<Output = Result<(), Error>> + 'static>>,
}

impl Unpin for IndradbConnLoop {}

impl Future for IndradbConnLoop {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {                
        let inner_ref = self.get_mut();
        let poll1 = inner_ref.rpc_system_driver.as_mut().poll(cx);
        match poll1 {
            Ready(res) => {
                return Ready(res);
            },
            Pending => {}
        };

        let poll2 = inner_ref.rpc_client_driver.as_mut().poll(cx);
        match poll2 {
            Ready(res) => {
                return Ready(res);
            },
            Pending => {
                return Pending;
            }
        };
    }
}

// impl IndradbConnLoop {
//     pub(super) fn new(conn: TcpStream, )
// }

