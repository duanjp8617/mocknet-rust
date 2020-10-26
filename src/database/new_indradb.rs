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
use crate::emunet::user;
use crate::autogen::service::Client as IndradbCapnpClient;
use super::new_message_queue::{Sender, Queue, create, error};
use crate::util::ClientTransaction;

use std::collections::HashMap;

type CapnpRpcDisconnector = Disconnector<Side>;
pub type IndradbClientError = error::MsgQError<capnp::Error>;

enum Request {
    Ping,
    Init(Vec<server::ContainerServer>),
}

enum Response {
    Ping(bool),
    Init(bool),
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

    async fn count_vertex_number(&self, vertex_type: &str) -> Result<usize, capnp::Error> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // query the vertex with vertex type
        let q = RangeVertexQuery::new(u32::MAX).t(Type::new(vertex_type).unwrap());
        ct.async_get_vertices(q).await.map(|v|{v.len()})
    }

    async fn create_vertex_json_value(&self, vertex_type: &str, property_name: &str, json_value: &serde_json::Value) 
        -> Result<bool, capnp::Error> 
    {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // create a new vertex with vertex_type
        let vt = Type::new(vertex_type).unwrap();
        let v = Vertex::new(vt);
        let succeed = ct.async_create_vertex(&v).await?;
        if !succeed {
            return Ok(succeed)
        }

        // create a new property with property_name
        let q = SpecificVertexQuery::single(v.id).property(property_name);
        ct.async_set_vertex_properties(q, json_value).await?;
        Ok(true)
    }

    async fn read_vertex_json_value(&self, vertex_type: &str, property_name: &str) 
        -> Result<serde_json::Value, capnp::Error> 
    {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // query the vertex with vertex type
        let q = RangeVertexQuery::new(1).t(Type::new(vertex_type).unwrap());
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            panic!("vertex type {} with property {} is not available in the database");
        }

        // read the value of property_name
        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() != 1 {
            panic!("vertex type {} with property {} is not available in the database");
        }

        Ok(property_list.pop().unwrap().value)
    }

    async fn write_vertex_json_value(&self, vertex_type: &str, property_name: &str, json_value: &serde_json::Value) 
        -> Result<(), capnp::Error> 
    {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        // get the server_list vertex
        let q = RangeVertexQuery::new(1).t(Type::new(vertex_type).unwrap());
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            panic!("vertex type {} with property {} is not available in the database");
        }

        // update the property for the queue
        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        ct.async_set_vertex_properties(q, json_value).await?;
        Ok(())
    }
}

impl IndradbClientBackend {
    async fn init(&self, servers: Vec<server::ContainerServer>) -> Result<bool, capnp::Error> {
        let c_server_list = self.count_vertex_number("server_list").await?;
        let c_user_map = self.count_vertex_number("user_map").await?;

        if c_server_list == 1 && c_user_map == 1 {        
            return Ok(false)
        }
        else if c_server_list == 0 && c_user_map == 0 {            
            let servers_jv = serde_json::to_value(servers).unwrap();
            let _succeed = self.create_vertex_json_value("server_list", "list", &servers_jv).await?;
            
            let users_jv = serde_json::to_value(HashMap::<String, user::EmuNetUser>::new()).unwrap();
            let _succeed = self.create_vertex_json_value("user_map", "map", &users_jv).await?;

            return Ok(true)
        }
        else {
            panic!("The database is incorrectly initialized, please check the database")
        }
    }

    async fn create_server_list(&self) -> Result<bool, capnp::Error> {        
        let json_value = serde_json::to_value(Vec::<server::ContainerServer>::new()).unwrap();
        self.create_vertex_json_value("server_list", "list", &json_value).await   
    }

    async fn read_server_list(&self) -> Result<Vec<server::ContainerServer>, capnp::Error> {
        let json_value = self.read_vertex_json_value("server_list", "list").await?;
        let server_list: Vec<server::ContainerServer> = serde_json::from_value(json_value).unwrap();
        Ok(server_list)
    }

    async fn update_server_list(&self, server_list: Vec<server::ContainerServer>) -> Result<(), capnp::Error> {
        let json_value = serde_json::to_value(server_list).unwrap();
        self.write_vertex_json_value("server_list", "list", &json_value).await
    }

    async fn create_user_map(&self) -> Result<bool, capnp::Error> {
        let json_value = serde_json::to_value(HashMap::<String, user::EmuNetUser>::new()).unwrap();
        self.create_vertex_json_value("user_map", "map", &json_value).await   
    }


}

impl IndradbClientBackend {
    async fn dispatch_request(&self, req: Request) -> Result<Response, capnp::Error> {
        match req {
            Request::Ping => {
                let res = self.ping().await?;
                Ok(Response::Ping(res))
            },
            Request::Init(servers) => {
                let res = self.init(servers).await?;
                Ok(Response::Init(res))
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
            _ => panic!("invalid response")
        }
    }

    pub async fn init(&self, servers: Vec<server::ContainerServer>) -> Result<bool, IndradbClientError> {
        let req = Request::Init(servers);
        let res = self.sender.send(req).await?;
        match res {
            Response::Init(res) => Ok(res),
            _ => panic!("invalid response")
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