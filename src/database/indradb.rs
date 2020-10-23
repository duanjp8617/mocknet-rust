// An implementation of Indradb storage backend
use std::future::Future;

use futures::AsyncReadExt;

use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use capnp_rpc::Disconnector;

use indradb::{RangeVertexQuery, SpecificVertexQuery, VertexQuery, VertexQueryExt, VertexPropertyQuery};
use indradb::Type;
use indradb::{Vertex};
use indradb::{VertexProperty};

use crate::emunet::server;
use crate::autogen::service::Client as IndradbCapnpClient;
use super::message_queue::{Sender, Queue, create};

type CapnpRpcDisconnector = Disconnector<Side>;
pub type IndradbClientError = super::message_queue::error::MsgQError;

enum Request {
    Wtf,
    Ping,
    ReadServers(String),
}

enum Response {
    Wtf,
    Ping(bool),
    ReadServers(Vec<server::ContainerServer>),
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

    // async fn read_server_list(&self, name: String) -> Result<Vec<server::ContainerServer>, capnp::Error> {
    //     // get the vertex of type "server_list"
    //     let q = RangeVertexQuery::new(1).t(Type::new("server_list").unwrap());
    //     let vertex_list = self.async_get_vertices(q.into()).await?;
    //     if vertex_list.len() != 1 {
    //         return Ok(Vec::new());
    //     }

    //     // get the "name" property of this vertex
    //     let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(name);
    //     let property_list = self.async_get_vertex_properties(q).await?;
    //     if property_list.len() != 1 {
    //         panic!("fatal, this should be impossible to reach");
    //     }
    //     let server_list: Vec<server::ContainerServer> = serde_json::from_value(property_list[0].value).unwrap();

    //     Ok(list.into_iter().map(|p|{serde_json::from_value(p.value).unwrap()}).collect())
    // }

    // async fn update_server_list(&self, name: String, server_list: Vec<server::ContainerServer>) -> Result<bool, capnp::Error> {

    // }
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
                    while let Ok(_) = queue.try_recv() {}
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
        println!("running core loop");
        let mut err_opt = None;        
        while let Some(mut msg) = queue.recv().await {
            println!("receive a message");
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
        println!("here");

        let disconnect_res = backend.disconnector.await;
        println!("Indradb network connection down");
        err_opt.map_or(disconnect_res, |err|{Err(err)})
    }
}

pub struct IndradbClient {
    sender: Sender<Request, Response>,
}

impl IndradbClient {
    pub async fn ping(&self) -> Result<bool, IndradbClientError> {
        let req = Request::Ping;
        println!("send ping request");
        let res = self.sender.send(req).await?;
        match res {
            Response::Ping(flag) => Ok(flag),
            _ => {
                panic!("invalid response type")
            }
        }
    }

    pub async fn get_server_pool(&self, name: String) -> Result<Vec<server::ContainerServer>, IndradbClientError> {
        let req = Request::ReadServers(name);
        let res = self.sender.send(req).await?;
        match res {
            Response::ReadServers(v) => Ok(v),
            _ => {
                panic!("invalid response type")
            }
        }
    }

    pub async fn update_server_pool(name: String, server_pool: Vec<server::ContainerServer>) -> Result<(), IndradbClientError> {
        unimplemented!()
    }

    // pub async fn user_registration(user: String) -> Result<(), IndradbClientError> {

    // }

}


pub fn build_client_fut<'a>(stream: tokio::net::TcpStream, ls: &'a tokio::task::LocalSet) 
    -> (IndradbClient, impl Future<Output = Result<(), capnp::Error>> + 'a)
{
    
    let (sender, queue) = create();

    let backend_fut = ls.run_until(async move {         
        // create rpc_system
        let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            Side::Client,
            Default::default(),
        ));
        let mut capnp_rpc_system = RpcSystem::new(rpc_network, None);
        
        // create client_backend
        let indradb_capnp_client = capnp_rpc_system.bootstrap(Side::Server);
        let disconnector = capnp_rpc_system.get_disconnector();
        let indradb_client_backend = IndradbClientBackend {
            client: indradb_capnp_client,
            disconnector,
        };

        // run rpc_system
        tokio::task::spawn_local(async move {
            capnp_rpc_system.await
        });
        // run indradb backend
        tokio::task::spawn_local(build_backend_fut(indradb_client_backend, queue)).await.unwrap()
    });
    
    (IndradbClient{sender}, backend_fut)
}
