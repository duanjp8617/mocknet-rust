// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use capnp_rpc::rpc_twoparty_capnp::Side;
use indradb::{RangeVertexQuery, SpecificVertexQuery, VertexQueryExt, VertexQuery};
use indradb::Type;
use indradb::{Vertex};
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::emunet::server;
use crate::emunet::user;
use crate::emunet::net;
use super::message_queue::{Queue};
use super::indradb_util::{ClientTransaction, generate_uuid_v1};
use super::errors::{BackendError};

// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const bytes_seed: [u8; 16] = [1, 2,  3,  4,  5,  6,  7,  8,
                              9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: Uuid = Uuid::from_bytes(bytes_seed);
}

/// A transaction worker that handles all the interaction with indradb.
struct TranWorker {
    client: crate::autogen::service::Client,
}

impl TranWorker {
    /// Create a vertex with an optional uuid.
    async fn create_vertex(&self, id: Option<Uuid>) -> Result<Uuid, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let t = Type::new("").unwrap();
        let v = match id {
            Some(id) => Vertex::with_id(id, t),
            None => Vertex::with_id(generate_uuid_v1(), t),
        };

        let succeed = ct.async_create_vertex(&v).await?;
        if succeed {
            Ok(v.id)
        }
        else {
            Err(BackendError::invalid_arg(format!("vertex {} already exists", &v.id)))
        }
    }

    /// Get json property with name `property_name` from vertex with id `vid`.
    async fn get_vertex_json_value(&self, vid: Uuid, property_name: &str) -> Result<serde_json::Value, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid.clone()).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            return Err(BackendError::invalid_arg(format!("vertex {} already exists", &vid)));
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() != 1 {
            return Err(BackendError::invalid_arg(format!("vertex has no property {}", property_name)));
        }

        Ok(property_list.pop().unwrap().value)
    }

    /// Set json property with name `property_name` for vertex with id `vid`.
    async fn set_vertex_json_value(&self, vid: Uuid, property_name: &str, json: &serde_json::Value) -> Result<(), BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            return Err(BackendError::invalid_arg(format!("vertex {} already exists", &vid)));
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        ct.async_set_vertex_properties(q, json).await.map_err(|e|{e.into()})
    }
}

#[derive(Clone)]
pub enum Request {
    Ping,
    Init(Vec<server::ContainerServer>),
    RegisterUser(String),
    CreateEmuNet(String, String, u32),
}

#[derive(Clone)]
pub enum Response {
    Ping(bool),
    Init(bool),
    RegisterUser(bool),
    CreateEmuNet(Uuid),
}

pub struct IndradbClientBackend {
    worker: TranWorker,
    disconnector: capnp_rpc::Disconnector<Side>,
}

impl IndradbClientBackend {
    pub fn new(client: crate::autogen::service::Client, disconnector: capnp_rpc::Disconnector<Side>) -> Self {
        Self{
            worker: TranWorker{client}, 
            disconnector
        }
    }

    // a few helper functions:
    async fn get_user_map(&self) -> Result<HashMap<String, user::EmuNetUser>, BackendError> {
        let jv = self.worker.get_vertex_json_value(Either::Left("user_map"), "map").await?;
        let user_map: HashMap<String, user::EmuNetUser> = serde_json::from_value(jv).unwrap();
        Ok(user_map)
    }

    async fn set_user_map(&self, user_map: HashMap<String, user::EmuNetUser>) -> Result<(), BackendError> {
        let jv = serde_json::to_value(user_map).unwrap();
        self.worker.update_vertex_json_value(Either::Left("user_map"), "map", &jv).await
    }

    async fn get_server_list(&self) -> Result<Vec<server::ContainerServer>, BackendError> {
        let jv = self.worker.read_vertex_json_value(Either::Left("server_list"), "list").await?;
        let server_list: Vec<server::ContainerServer> = serde_json::from_value(jv).unwrap();
        Ok(server_list)
    }

    async fn set_server_list(&self, server_list: Vec<server::ContainerServer>) -> Result<(), BackendError> {
        let jv = serde_json::to_value(server_list).unwrap();
        self.worker.update_vertex_json_value(Either::Left("server_list"), "mlist", &jv).await
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
        let mut user_map = self.get_user_map().await?;
        if user_map.get(&user_name).is_some() {
            return Ok(false);
        }

        let user = user::EmuNetUser::new(&user_name);
        user_map.insert(user_name, user);        
        self.set_user_map(user_map).await.map(|_|{true})
    }

    async fn create_emu_net(&self, user: String, net: String, capacity: u32) -> Result<Uuid, BackendError> {
        let mut user_map = self.get_user_map().await?;
        if user_map.get(&user).is_none() {
            return Err(BackendError::invalid_arg("invalid user name".to_string()));
        }
        let user_mut = user_map.get_mut(&user).unwrap();

        // check whether the net has existed
        if user_mut.emu_net_exist(&net) {
            return Err(BackendError::invalid_arg("invalid emu-net name".to_string()));
        }

        // TODO: update server pool API and make the code shorter.
        let server_list = self.get_server_list().await?;
        let mut sp = server::ServerPool::new();
        sp.add_servers(server_list.into_iter());
        let allocated_opt = sp.allocate_servers(capacity);
        self.set_server_list(sp.into_vec()).await?;
        if allocated_opt.is_none() {
            return Err(BackendError::invalid_arg("invalid capacity".to_string()));
        }

        // create a new emu net
        let mut emu_net = net::EmuNet::new(net.clone(), capacity);
        emu_net.add_servers(allocated_opt.unwrap());
        // create a new emu net node
        let emu_net_id = self.worker.create_vertex(None, &format!("{}_{}", &user, &net)).await?;
        // write the new emu_net property into the new node
        let jv = serde_json::to_value(emu_net).unwrap();
        self.worker.update_vertex_json_value(Either::Right(emu_net_id.clone()), "default", &jv).await?;

        // write the user_map property into the new node
        user_mut.add_emu_net(net, emu_net_id.clone());
        self.set_user_map(user_map).await?;

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
            },
            Request::CreateEmuNet(user, net, capacity) => {
                let res = self.create_emu_net(user, net, capacity).await?;
                Ok(Response::CreateEmuNet(res))
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
