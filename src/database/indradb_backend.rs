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

    async fn find_vertex(&self, id: Uuid) -> Result<bool, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        let q = SpecificVertexQuery::single(id.clone());
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            Ok(false)
        }
        else if vertex_list.len() == 1 {
            Ok(true)
        }
        else {
            Err(BackendError::invalid_arg("too many vertexes".to_string()))
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

    async fn create_emu_net(&self, user: String, net: String, capacity: u32) -> Result<Uuid, BackendError> {
        let jv = self.worker.read_vertex_json_value(Either::Left("user_map"), "map").await?;
        let mut user_map: HashMap<String, user::EmuNetUser> = serde_json::from_value(jv).unwrap();
        if user_map.get(&user).is_none() {
            return Err(BackendError::invalid_arg("invalid user name".to_string()));
        }
        let user_mut = user_map.get_mut(&user).unwrap();

        // check whether the net has existed
        if user_mut.emu_net_exist(&net) {
            return Err(BackendError::invalid_arg("invalid emu-net name".to_string()));
        }

        // TODO: update server pool API and make the code shorter.
        let jv = self.worker.read_vertex_json_value(Either::Left("server_list"), "list").await?;
        let server_list: Vec<server::ContainerServer> = serde_json::from_value(jv).unwrap();
        let mut sp = server::ServerPool::new();
        sp.add_servers(server_list.into_iter());
        let allocated_opt = sp.allocate_servers(capacity);
        let servers_jv = serde_json::to_value(sp.into_vec()).unwrap();
        self.worker.update_vertex_json_value(Either::Left("server_list"), "list", &servers_jv).await?;
        if allocated_opt.is_none() {
            return Err(BackendError::invalid_arg("invalid capacity".to_string()));
        }

        // create a new emu net
        let mut emu_net = net::EmuNet::new(net.clone(), capacity);
        emu_net.add_servers(allocated_opt.unwrap());
        // create a new emu net node
        let emu_net_id = self.worker.create_vertex(None, &format!("{}:{}", &user, &net)).await?;
        // write the new emu_net property into the new node
        let jv = serde_json::to_value(emu_net).unwrap();
        self.worker.update_vertex_json_value(Either::Right(emu_net_id.clone()), "default", &jv).await?;

        // write the user_map property into the new node
        user_mut.add_emu_net(net, emu_net_id.clone());
        let jv = serde_json::to_value(user_map).unwrap();
        self.worker.update_vertex_json_value(Either::Left("user_map"), "map", &jv).await?;

        Ok(emu_net_id)
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
