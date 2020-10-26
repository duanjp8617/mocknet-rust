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

use uuid::Uuid;

use crate::emunet::server;
use crate::autogen::service::Client as IndradbCapnpClient;
use super::new_message_queue::{Sender, Queue, create, error};
use crate::util::ClientTransaction;

type CapnpRpcDisconnector = Disconnector<Side>;
pub type IndradbClientError = error::MsgQError<capnp::Error>;

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

    async fn create_server_list(&self) -> Result<bool, capnp::Error> {        
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // create the server_list vertex
        let vt = Type::new("server_list").unwrap();
        let v = Vertex::new(vt);
        // return true if succeed, if the vertex already exists, return false 
        ct.async_create_vertex(&v).await        
    }

    async fn read_server_list(&self, name: String) -> Result<Vec<server::ContainerServer>, capnp::Error> {
        // get the vertex of type "server_list"
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // get the server_list vertex
        let q = RangeVertexQuery::new(1).t(Type::new("server_list").unwrap());
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            // if server_list vertex is not available, return an empty list
            return Ok(Vec::new())
        }

        // get the "name" property of this vertex
        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() != 1 {
            // if property_list is not available, return an empty list
            return Ok(Vec::new())
        }

        let json_value = property_list.pop().unwrap().value;
        let server_list: Vec<server::ContainerServer> = serde_json::from_value(json_value).unwrap();
        Ok(server_list)
    }

    async fn update_server_list(&self, name: String, server_list: Vec<server::ContainerServer>) -> Result<bool, capnp::Error> {
        // get the vertex of type "server_list"
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // get the server_list vertex
        let q = RangeVertexQuery::new(1).t(Type::new("server_list").unwrap());
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            // if server_list vertex is not available, return an empty list
            return Ok(false)
        }

        // update the property for the queue
        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(name);
        let json_value = serde_json::to_value(server_list).unwrap();
        ct.async_set_vertex_properties(q, &json_value).await?;
        Ok(true)
    }

    // create a new vertex with type "user_list", initialzie "user_map" property
    // containing a map between user name and user struct
    async fn create_user_list(&self) -> Result<bool, capnp::Error> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // create the server_list vertex
        let vt = Type::new("user_list").unwrap();
        let v = Vertex::new(vt);
        // return true if succeed, if the vertex already exists, return false 
        let res = ct.async_create_vertex(&v).await?;
        // If the vertex is already there, 
        if res == false {
            return Ok(false);
        }

        // update the property for the queue
        let q = SpecificVertexQuery::new(vec!(v.id)).property("list");
        let json_value = serde_json::to_value(Vec::<i32>::new()).unwrap();
        ct.async_set_vertex_properties(q, &json_value).await?;
        Ok(true)
    }

    // check if the user_map contains the name, if so, the user has already 
    // been registered, the registration fails, return false.
    // otherwise, create a new entry in the map, finish registration and return true.
    async fn register_new_user(&self, name: String) -> Result<bool, capnp::Error> {
        unimplemented!()
    }

    // 
    async fn create_enet(&self, name: String) -> Result<Option<Uuid>, capnp::Error> {

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

fn build_backend_fut(backend: IndradbClientBackend, mut queue: Queue<Request, Response, capnp::Error>) 
    -> impl Future<Output = Result<(), capnp::Error>> + 'static 
{
    fn drain_queue(mut queue: Queue<Request, Response, capnp::Error>) {
        queue.close();
        while let Ok(_) = queue.try_recv() {}
    }
    
    async move {        
        while let Some(mut msg) = queue.recv().await {
            if msg.is_close_msg() {
                drain_queue(queue);
                break;
            }
            else {                
                let req = msg.try_get_msg().unwrap();
                let resp_result = backend.dispatch_request(req).await;
                let _ = msg.callback(resp_result);
            }
        }
        
        backend.disconnector.await        
    }
}

pub struct IndradbClient {
    sender: Sender<Request, Response, capnp::Error>,
}

impl IndradbClient {
    pub async fn ping(&self) -> Result<bool, IndradbClientError> {
        let req = Request::Ping;
        let res = self.sender.send(req).await?;
        match res {
            Response::Ping(flag) => Ok(flag),
            _ => {
                panic!("invalid response")
            }
        }
    }
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