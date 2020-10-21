// An implementation of Indradb storage backend
use std::future::Future;
use std::net::SocketAddr;

use futures::AsyncReadExt;

use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use capnp_rpc::Disconnector;

use crate::autogen;
use super::message_queue1::{self, Queue};

use autogen::service::Client as IndradbCapnpClient;
type CapnpRpcSystem = RpcSystem<Side>;
type CapnpRpcDisconnector = Disconnector<Side>;

// IndradbConnector
struct Connector {
    addr: SocketAddr,
}

impl Connector {
    async fn connect(self) -> Result<(CapnpRpcSystem, IndradbClientBackend), std::io::Error> {
        // make a connection
        let stream = tokio::net::TcpStream::connect(&self.addr).await?;    
        stream.set_nodelay(true)?;

         
        // create rpc_system
        let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            Side::Client,
            Default::default(),
        ));
        let mut capnp_rpc_system = RpcSystem::new(rpc_network, None);
        
        // create client_wrapper
        let indradb_capnp_client = capnp_rpc_system.bootstrap(Side::Server);
        let disconnector = capnp_rpc_system.get_disconnector();
        let indradb_client_backend = IndradbClientBackend {
            client: indradb_capnp_client,
            disconnector,
        };

        Ok((capnp_rpc_system, indradb_client_backend))
    }   
}

struct IndradbClientBackend {
    client: IndradbCapnpClient,
    disconnector: CapnpRpcDisconnector,
}

impl IndradbClientBackend {
    async fn ping(&self) -> Result<bool, capnp::Error> {
        let req = self.client.ping_request();
        let res = req.send().promise.await?;
        Ok(res.get()?.get_ready()) 
    }
}

impl IndradbClientBackend {
    async fn dispatch_request(&self, req: Request) -> Result<Response, capnp::Error> {
        match req {
            Request::Ping => {
                let resp = self.ping().await?;
                Ok(Response::Ping(resp))
            },
            _ => {
                panic!("wtf?")
            }
        }
    }
}

fn build_backend_fut(backend: IndradbClientBackend, mut queue: Queue<Request, Response>) 
    -> impl Future<Output = Result<(), capnp::Error>> + 'static 
{
    async fn shutdown_queue(backend: &IndradbClientBackend, mut queue: Queue<Request, Response>) 
        -> Result<(), capnp::Error> 
    {
        queue.close();
        while let Ok(mut msg) = queue.try_recv() {
            let req = msg.try_get_msg().expect("find another error message");
            let resp_result = backend.dispatch_request(req).await;
            match resp_result {
                Ok(resp) => {
                    let _ = msg.callback(resp);
                },
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    fn drain_queue(mut queue: Queue<Request, Response>) {
        queue.close();
        while let Ok(_) = queue.try_recv() {}
    }
    
    async move {
        let mut err_opt = None;        
        while let Some(mut msg) = queue.recv().await {
            if msg.is_close_msg() {
                err_opt = shutdown_queue(&backend, queue).await.err();
                break;
            }
            else {                
                let req = msg.try_get_msg().unwrap();
                let resp_result = backend.dispatch_request(req).await;
                match resp_result {
                    Ok(resp) => {
                        let _ = msg.callback(resp);
                    },
                    Err(err) => {
                        drain_queue(queue);
                        err_opt = Some(err);
                        break;
                    }

                }
            }
        }

        let disconnect_res = backend.disconnector.await;
        err_opt.map_or(disconnect_res, |err|{Err(err)})
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
    sender: message_queue1::Sender<Request, Response>,
}

impl IndradbClient {
    pub async fn ping(&self) -> Result<bool, message_queue1::error::MsgQError> {
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

impl IndradbClient {
    async fn build(addr: SocketAddr, ls: tokio::task::LocalSet) -> Result<Self, std::io::Error> {
        let (sender, queue) = message_queue1::create();

        ls.run_until(async move {            
            let (capnp_rpc_system, indradb_client_backend) = Connector{addr}.connect().await?;

            tokio::task::spawn_local(async move {
                capnp_rpc_system.await
            });
            tokio::task::spawn_local(build_backend_fut(indradb_client_backend, queue));

            Result::<(), std::io::Error>::Ok(())           
        }).await?;
        
        Ok(Self {
            sender
        })
    }
}