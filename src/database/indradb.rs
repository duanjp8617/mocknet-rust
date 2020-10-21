// An implementation of Indradb storage backend
use std::future::Future;
use std::task::{Context, Poll, Poll::Ready, Poll::Pending};
use std::pin::Pin;
use std::marker::Unpin;
use std::net::SocketAddr;

use futures::AsyncReadExt;

use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};

use crate::errors::Error;
use crate::autogen;
use super::message_queue;

// A request sent from the database client
enum Request {
    Wtf,
    Ping,
}

enum Response {
    Wtf,
    Ping(bool),
}

struct IndradbCapnpClient {
    client: autogen::service::Client,
}

impl IndradbCapnpClient {
    async fn ping(&self) -> Result<bool, Error> {
        let req = self.client.ping_request();
        let res = req.send().promise.await?;
        Ok(res.get()?.get_ready()) 
    }
}

impl IndradbCapnpClient {
    fn build_driver(self, mut queue: message_queue::Queue<Request, Response>) -> impl Future<Output = Result<(), Error>> + 'static {
        // the core loop for running capnp rpc
        async move {
            println!("inside core");
            while let Some(msg) = queue.recv().await {
                println!("receive ping request");
                let (req, cb_tx) = msg.take_inner();
                
                match req {
                    Request::Ping => {                        
                        let resp = self.ping().await?;
                        let _ = cb_tx.send(Response::Ping(resp));
                    }
                    _ => {
                        panic!("Wtf")
                    }
                }
            }
    
            Ok(())
        }
    }
}

pub struct IndradbClient {
    sender: message_queue::Sender<Request, Response>,
}

impl IndradbClient {
    pub async fn ping(&self) -> Result<bool, message_queue::error::SenderError> {
        let req = Request::Ping;
        println!("send ping request");
        let res = self.sender.send(req).await?;
        match res {
            Response::Ping(flag) => Ok(flag),
            _ => {
                panic!("wtf")
            }
        }
    }
}

pub struct IndradbConnLoop {
    capnp_rpc_driver: Pin<Box<dyn Future<Output = Result<(), Error>> + 'static>>,
    capnp_client_driver: Pin<Box<dyn Future<Output = Result<(), Error>> + 'static>>,
}

impl Unpin for IndradbConnLoop {}

impl Future for IndradbConnLoop {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {       
        println!("polling?");
        let inner_ref = self.get_mut();
        let poll1 = inner_ref.capnp_rpc_driver.as_mut().poll(cx);
        match poll1 {
            Ready(res) => {
                return Ready(res);
            },
            Pending => {}
        };

        let poll2 = inner_ref.capnp_client_driver.as_mut().poll(cx);
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

pub async fn build(addr: &SocketAddr) -> Result<(IndradbClient, IndradbConnLoop), Error> {
    // Make a connection
    let stream = tokio::net::TcpStream::connect(addr).await?;
    println!("connection ready");   
    stream.set_nodelay(true)?;
 
    // create rpc_network
    let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
    let rpc_network = Box::new(twoparty::VatNetwork::new(
        reader,
        writer,
        Side::Client,
        Default::default(),
    ));

    // create capnp_rpc_system and indradb_capnp_client
    let mut capnp_rpc_system = RpcSystem::new(rpc_network, None);
    let indradb_capnp_client = IndradbCapnpClient {
        client: capnp_rpc_system.bootstrap(Side::Server),
    };

    // create message queue
    let (sender, queue) = message_queue::create();

    // create capnp drivers
    let capnp_rpc_driver = async move {
        println!("running capnp_rpc_system");
        capnp_rpc_system.await.map_err(|e| {e.into()})
    };
    let capnp_client_driver = IndradbCapnpClient::build_driver(indradb_capnp_client, queue);

    // build connection loop
    let conn_loop = IndradbConnLoop {
        capnp_rpc_driver: Box::pin(capnp_rpc_driver),
        capnp_client_driver: Box::pin(capnp_client_driver)
    };

    Ok((IndradbClient{sender}, conn_loop))
}