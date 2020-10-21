// An implementation of Indradb storage backend
use std::future::Future;
use std::task::{Context, Poll, Poll::Ready, Poll::Pending};
use std::pin::Pin;
use std::marker::Unpin;
use std::net::SocketAddr;

use futures::AsyncReadExt;

use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use capnp_rpc::{self, rpc_twoparty_capnp};

use crate::errors::Error;
use crate::autogen;
use super::message_queue;

use autogen::service::Client as IndradbCapnpClient;
type CapnpRpcSystem = RpcSystem<Side>;

// IndradbConnector
struct Connector {
    addr: SocketAddr,
}

impl Connector {
    async fn connect(self) -> Result<(CapnpRpcSystem, IndradbCapnpClient), std::io::Error> {
        // make a connection
        let stream = tokio::net::TcpStream::connect(&self.addr).await?;    
        stream.set_nodelay(true)?;

         
        // create rpc_system and capnp_client
        let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            Side::Client,
            Default::default(),
        ));
        let mut capnp_rpc_system = RpcSystem::new(rpc_network, None);
        let indradb_capnp_client = capnp_rpc_system.bootstrap(Side::Server);

        Ok((capnp_rpc_system, indradb_capnp_client))
    }   
}

struct ClientWrapper {
    client: IndradbCapnpClient,
}

impl ClientWrapper {
    async fn ping(&self) -> Result<bool, Error> {
        let req = self.client.ping_request();
        let res = req.send().promise.await?;
        Ok(res.get()?.get_ready()) 
    }
}

impl ClientWrapper {
    fn build_driver(self, mut queue: message_queue::Queue<Request, Response>) -> impl Future<Output = Result<(), Error>> + 'static {
        // the core loop for running capnp rpc
        async move {
            while let Some(msg) = queue.recv().await {
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

enum Request {
    Wtf,
    Ping,
}

enum Response {
    Wtf,
    Ping(bool),
}

struct IndradbClient {
    sender: message_queue::Sender<Request, Response>,
}

// impl IndradbClient {
//     async fn build(addr: SocketAddr) -> Result<Self, Error> {
//         let (sender, queue) = message_queue::create();

//         let jh = tokio::spawn(async move {
//             let (capnp_rpc_system, indradb_capnp_client) = Connector::new(addr).connect().await?;

//             let ls = tokio::task::LocalSet::new();
//             ls.run_until(async move {
//                 tokio::task::spawn_local(async move {
//                     capnp_rpc_system.await
//                 });

//                 tokio::task::spawn_local(async move {

//                 })

//             }).await;

//             Ok(())
//         });
//     }
// }