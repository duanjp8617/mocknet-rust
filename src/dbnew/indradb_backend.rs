// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use capnp_rpc::rpc_twoparty_capnp::Side;
use indradb::{SpecificVertexQuery, VertexQueryExt, VertexQuery};
use indradb::{Vertex, Type};
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::emunet::server;
use crate::emunet::user;
use crate::emunet::net;
use super::message_queue::{Queue};
use super::indradb_util::ClientTransaction;
use super::errors::{BackendError, BackendErrorKind};

// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const BYTES_SEED: [u8; 16] = [1, 2,  3,  4,  5,  6,  7,  8,
                              9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: Uuid = Uuid::from_bytes(BYTES_SEED);
}

/// A transaction worker that handles all the interaction with indradb.
struct TranWorker {
    client: crate::autogen::service::Client,
}

impl TranWorker {
    // this should be removed
    async fn ping(&self) -> Result<bool, BackendError> {
        let req = self.client.ping_request();
        let res = req.send().promise.await?;
        Ok(res.get()?.get_ready()) 
    }

    // create a vertex with an optional uuid
    async fn create_vertex(&self, id: Option<Uuid>) -> Result<Uuid, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let t = Type::new("").unwrap();
        let v = match id {
            Some(id) => Vertex::with_id(id, t),
            None => Vertex::with_id(indradb::util::generate_uuid_v1(), t),
        };

        let succeed = ct.async_create_vertex(&v).await?;
        if succeed {
            Ok(v.id)
        }
        else {
            Err(BackendError::data_error(format!("vertex {} exists", v.id)))
        }
    }

    // get json property with name `property_name` from vertex with id `vid`
    async fn get_vertex_json_value(&self, vid: Uuid, property_name: &str) -> Result<serde_json::Value, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid.clone()).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            panic!("indradb fatal error");
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() != 1 {
            panic!("indradb fatal error");
        }

        Ok(property_list.pop().unwrap().value)
    }

    // set json property with name `property_name` for vertex with id `vid`
    async fn set_vertex_json_value(&self, vid: Uuid, property_name: &str, json: &serde_json::Value) -> Result<(), BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() != 1 {
            panic!("indradb fatal error");
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
    Init,
    RegisterUser,
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
        let jv = self.worker.get_vertex_json_value(CORE_INFO_ID.clone(), "user_map").await?;
        let user_map: HashMap<String, user::EmuNetUser> = serde_json::from_value(jv).unwrap();
        Ok(user_map)
    }

    async fn set_user_map(&self, user_map: HashMap<String, user::EmuNetUser>) -> Result<(), BackendError> {
        let jv = serde_json::to_value(user_map).unwrap();
        self.worker.set_vertex_json_value(CORE_INFO_ID.clone(), "user_map", &jv).await
    }

    async fn get_server_list(&self) -> Result<Vec<server::ContainerServer>, BackendError> {
        let jv = self.worker.get_vertex_json_value(CORE_INFO_ID.clone(), "server_list").await?;
        let server_list: Vec<server::ContainerServer> = serde_json::from_value(jv).unwrap();
        Ok(server_list)
    }

    async fn set_server_list(&self, server_list: Vec<server::ContainerServer>) -> Result<(), BackendError> {
        let jv = serde_json::to_value(server_list).unwrap();
        self.worker.set_vertex_json_value(CORE_INFO_ID.clone(), "server_list", &jv).await
    }
}

impl IndradbClientBackend {
    // this should be removed
    async fn ping(&self) -> Result<bool, BackendError> {
        self.worker.ping().await
    }

    async fn init(&self, servers: Vec<server::ContainerServer>) -> Result<(), BackendError> {
        let res = self.worker.create_vertex(Some(CORE_INFO_ID.clone())).await;
        match res {
            Ok(_) => {
                // initialize user map
                self.set_user_map(HashMap::<String, user::EmuNetUser>::new()).await?;

                // initialize server list                
                self.set_server_list(servers).await?;
                        
                Ok(())
            },
            Err(e) => {
                // e may be a DataError due to intialized database
                // this is not an error and change it to InvalidArg
                match e.kind() {
                    BackendErrorKind::DataError => Err(BackendError::invalid_arg("the database has initialized".to_string())),
                    _ => Err(e)
                }
            },
        }
    }

    async fn register_user(&self, user_id: String) -> Result<(), BackendError> {        
        // read current user map
        let mut user_map = self.get_user_map().await?;
        if user_map.get(&user_id).is_some() {
            return Err(BackendError::invalid_arg("user has registered".to_string()));
        }

        // register the new user
        let user = user::EmuNetUser::new(&user_id);
        user_map.insert(user_id, user);        
        
        // sync update in the db
        self.set_user_map(user_map).await
    }

    async fn create_emu_net(&self, user: String, net: String, capacity: u32) -> Result<Uuid, BackendError> {
        // get the user
        let mut user_map = self.get_user_map().await?;
        if user_map.get(&user).is_none() {
            return Err(BackendError::invalid_arg("invalid user name".to_string()));
        }
        let user_mut = user_map.get_mut(&user).unwrap();

        // check whether the emunet has existed
        if user_mut.emu_net_exist(&net) {
            return Err(BackendError::invalid_arg("invalid emu-net name".to_string()));
        }

        // get the allocation of servers
        let server_list = self.get_server_list().await?;
        let mut sp = server::ServerPool::from(server_list);
        let allocation = match sp.allocate_servers(capacity) {
            Some(alloc) => alloc,
            None => return Err(BackendError::invalid_arg("invalid capacity".to_string())),
        };
        self.set_server_list(sp.into_vec()).await?;


        // create a new emu net
        let mut emu_net = net::EmuNet::new(net.clone(), capacity);
        emu_net.add_servers(allocation);
        
        // create and initialize a new emu net node
        let emu_net_id = self.worker.create_vertex(None).await?;
        let jv = serde_json::to_value(emu_net).unwrap();
        self.worker.set_vertex_json_value(emu_net_id, "default", &jv).await?;

        // add the new emunet to user map
        user_mut.add_emu_net(net, emu_net_id.clone());
        self.set_user_map(user_map).await?;

        Ok(emu_net_id)
    }
}

impl IndradbClientBackend {
    async fn dispatch_request(&self, req: Request) -> Result<Response, BackendError> {
        match req {
            Request::Ping => {
               self.ping().await.map(|succeed|{ Response::Ping(succeed)})
            },
            Request::Init(servers) => {
                self.init(servers).await.map(|_|{Response::Init})
            },
            Request::RegisterUser(user_name) => {
                self.register_user(user_name).await.map(|_|{Response::RegisterUser})                
            },
            Request::CreateEmuNet(user, net, capacity) => {
                self.create_emu_net(user, net, capacity).await.map(|id|{Response::CreateEmuNet(id)})                
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
