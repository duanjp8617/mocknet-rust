// An implementation of Indradb storage backend
use std::future::Future;

use capnp_rpc::rpc_twoparty_capnp::Side;

use indradb::{RangeVertexQuery, SpecificVertexQuery, VertexQueryExt, VertexQuery};
use indradb::Type;
use indradb::{Vertex};

use uuid::Uuid;

use crate::emunet::server;
use crate::emunet::user;
use crate::emunet::net;
use super::message_queue::{Queue};
use super::indradb_util::ClientTransaction;
use super::errors::{BackendError};

use std::collections::HashMap;

enum Either<L, R> {
    Left(L),
    Right(R),
}

struct IndradbTransactionWorker {
    client: crate::autogen::service::Client,
}

impl IndradbTransactionWorker {

    async fn ping(&self) -> Result<bool, BackendError> {
        let req = self.client.ping_request();
        let res = req.send().promise.await?;
        Ok(res.get()?.get_ready()) 
    }

    async fn count_vertex_number(&self, vertex_type: &str) -> Result<usize, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q = RangeVertexQuery::new(u32::MAX).t(Type::new(vertex_type).unwrap());
        let ls = ct.async_get_vertices(q).await?;
        Ok(ls.len())
    }

    async fn create_vertex(&self, id: Option<Uuid>, vt: &str) -> Result<Uuid, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let t = Type::new(vt).unwrap();
        let v = match id {
            Some(id) => Vertex::with_id(id, t),
            None => Vertex::new(t),
        };

        let succeed = ct.async_create_vertex(&v).await?;
        if succeed {
            Ok(v.id)
        }
        else {
            Err(BackendError::invalid_arg(format!("vertex of type {} exists", vt)))
        }
    }

    async fn read_vertex_json_value(&self, vertex_info: Either<&str, Uuid>, property_name: &str) -> Result<serde_json::Value, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = match vertex_info {
            Either::Left(vt) => RangeVertexQuery::new(1).t(Type::new(vt).unwrap()).into(),
            Either::Right(id) => SpecificVertexQuery::single(id.clone()).into(),
        };
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Err(BackendError::invalid_arg("vertex does not exist".to_string()));
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() == 0 {
            return Err(BackendError::invalid_arg(format!("vertex has no property {}", property_name)));
        }

        Ok(property_list.pop().unwrap().value)
    }

    async fn update_vertex_json_value(&self, vertex_info: Either<&str, Uuid>, property_name: &str, json: &serde_json::Value) -> Result<(), BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = match vertex_info {
            Either::Left(vt) => RangeVertexQuery::new(1).t(Type::new(vt).unwrap()).into(),
            Either::Right(id) => SpecificVertexQuery::single(id).into(),
        };
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Err(BackendError::invalid_arg("vertex does not exist".to_string()));
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        ct.async_set_vertex_properties(q, json).await.map_err(|e|{e.into()})
    }
}

pub enum Request {
    Ping,
    Init(Vec<server::ContainerServer>),
    RegisterUser(String),
}

pub enum Response {
    Ping(bool),
    Init(bool),
    RegisterUser(bool),
}

pub struct IndradbClientBackend {
    worker: IndradbTransactionWorker,
    disconnector: capnp_rpc::Disconnector<Side>,
}

impl IndradbClientBackend {
    pub fn new(client: crate::autogen::service::Client, disconnector: capnp_rpc::Disconnector<Side>) -> Self {
        Self{
            worker: IndradbTransactionWorker{client}, 
            disconnector
        }
    }
}

impl IndradbClientBackend {
    async fn ping(&self) -> Result<bool, BackendError> {
        self.worker.ping().await
    }

    async fn init(&self, servers: Vec<server::ContainerServer>) -> Result<bool, BackendError> {
        let c_server_list = self.worker.count_vertex_number("server_list").await?;
        let c_user_map = self.worker.count_vertex_number("user_map").await?;

        if c_server_list == 1 && c_user_map == 1 {        
            Ok(false)
        }
        else if c_server_list == 0 && c_user_map == 0 {  
            let server_list_id = self.worker.create_vertex(None, "server_list").await?;
            let servers_jv = serde_json::to_value(servers).unwrap();
            self.worker.update_vertex_json_value(Either::Right(server_list_id), "list", &servers_jv).await?;
                        
            let user_map_id = self.worker.create_vertex(None, "user_map").await?;
            let users_jv = serde_json::to_value(HashMap::<String, user::EmuNetUser>::new()).unwrap();
            self.worker.update_vertex_json_value(Either::Right(user_map_id), "map", &users_jv).await?;

            Ok(true)
        }
        else {
            Err(BackendError::invalid_arg("FATAL: database is polluted".to_string()))
        }
    }

    async fn register_user(&self, user_name: String) -> Result<bool, BackendError> {
        let jv = self.worker.read_vertex_json_value(Either::Left("user_map"), "map").await?;
        let mut user_map: HashMap<String, user::EmuNetUser> = serde_json::from_value(jv).unwrap();
        if user_map.get(&user_name).is_some() {
            return Ok(false);
        }

        let user = user::EmuNetUser::new(&user_name);
        user_map.insert(user_name, user);
        let jv = serde_json::to_value(user_map).unwrap();
        self.worker.update_vertex_json_value(Either::Left("user_map"), "map", &jv).await.map(|_|{true})
    }
}

impl IndradbClientBackend {
    async fn dispatch_request(&self, req: Request) -> Result<Response, BackendError> {
        match req {
            Request::Ping => {
                let res = self.ping().await?;
                Ok(Response::Ping(res))
            },
            Request::Init(servers) => {
                let res = self.init(servers).await?;
                Ok(Response::Init(res))
            },
            Request::RegisterUser(user_name) => {
                let res = self.register_user(user_name).await?;
                Ok(Response::RegisterUser(res))
            }
        }
    }
}

pub fn build_backend_fut(backend: IndradbClientBackend, mut queue: Queue<Request, Response, BackendError>) 
    -> impl Future<Output = Result<(), BackendError>> + 'static 
{
    fn drain_queue(mut queue: Queue<Request, Response, BackendError>) {
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
                let resp_result = backend.dispatch_request(req).await.map_err(|e|{e.into()});
                let _ = msg.callback(resp_result);
            }
        }
        
        backend.disconnector.await.map_err(|e|{e.into()})
    }
}
