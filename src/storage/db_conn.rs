use crate::autogen;
use crate::errors::Error;

use std::net::{ToSocketAddrs, SocketAddr};

use futures::AsyncReadExt;
use futures::FutureExt;

use std::future::Future;
use std::task::{Context, Poll, Poll::Ready, Poll::Pending};

use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};

use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::oneshot;

use std::pin::Pin;
use std::marker::Unpin;

enum DBRequest {
    Ping,
}

// struct IndradbBackend {
//     rpc_system: RpcSystem<Side>,
//     rpc_client: autogen::service::Client,
// }

pub struct DBConnLoop {
    rpc_system_driver: Pin<Box<dyn Future<Output = Result<(), Error>> + 'static>>,
    rpc_client_driver: Pin<Box<dyn Future<Output = Result<(), Error>> + 'static>>,
}

impl Unpin for DBConnLoop {}

impl Future for DBConnLoop {
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

#[derive(Clone)]
pub struct DBReqSender {
    inner: UnboundedSender<DBRequest>,
}

impl DBReqSender {
    pub fn ping(&mut self) {
        let res = self.inner.send(DBRequest::Ping);
        if res.is_err() {
            panic!("wtf?");
        }
    }
}

pub struct DBConn {
    rpc_system: RpcSystem<Side>,
    rpc_client: autogen::service::Client,
}

impl DBConn {
    pub async fn new(addr: &SocketAddr) -> Result<Self, Error> {
        // Make a connection
        let stream = tokio::net::TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;
        
        // Split the stream into reader and write, then create rpc_network
        let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            Side::Client,
            Default::default(),
        ));

        // Create the eventual DBConn
        let mut rpc_system = RpcSystem::new(rpc_network, None);
        let rpc_client: autogen::service::Client = rpc_system.bootstrap(Side::Server);
        Ok(Self{rpc_system, rpc_client})
    }

    pub fn launch(self) -> (DBReqSender, DBConnLoop) {
        let (sender, mut receiver) = unbounded_channel();
        let rpc_system = self.rpc_system;
        let rpc_client = self.rpc_client;

        let req_sender = DBReqSender {
            inner: sender,
        };

        let rpc_system_driver = async move {
            rpc_system.await.map_err(|e| {e.into()})
        };

        let rpc_client_driver = async move {
            let client = DBClient { client: rpc_client };
            
            while let Some(item) = receiver.recv().await {
                println!("receive a message");
                let res = client.ping().await?;
            }

            Ok(())
        };

        let conn_loop = DBConnLoop {
            rpc_system_driver: Box::pin(rpc_system_driver),
            rpc_client_driver: Box::pin(rpc_client_driver)
        };

        (req_sender, conn_loop)
    }
}

struct DBClient {
    client: autogen::service::Client,
}

impl DBClient {
    fn new(client: autogen::service::Client) -> Self {
        Self { client }
    }

    async fn ping(&self) -> Result<(), Error> {
        let req = self.client.ping_request();
        let res = req.send().promise.await?;
        if res.get()?.get_ready() {
            println!("ping ok");
            Ok(())
        } else {
            println!("ping err");
            Err(Error::capnp_error("ping returns false".to_string()))
        }
    }
}
