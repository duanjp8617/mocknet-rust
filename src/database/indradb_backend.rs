// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use capnp_rpc::rpc_twoparty_capnp::Side;
use indradb::{SpecificVertexQuery, VertexQueryExt, VertexQuery};
use indradb::{Vertex, Type};
use uuid::Uuid;
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Serialize};

use crate::emunet::server;
use crate::emunet::user;
use crate::emunet::net;
use super::message_queue::{Queue};
use super::indradb_util::ClientTransaction;
use super::errors::{BackendError};
use super::{QueryResult, QueryOk, QueryFail};

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
    // create a vertex with an optional uuid
    async fn create_vertex(&self, id: Option<Uuid>) -> Result<Option<Uuid>, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let t = Type::new("f").unwrap();
        let v = match id {
            Some(id) => Vertex::with_id(id, t),
            None => Vertex::with_id(indradb::util::generate_uuid_v1(), t),
        };

        let succeed = ct.async_create_vertex(&v).await?;
        if succeed {
            Ok(Some(v.id))
        }
        else {
            Ok(None)
        }
    }

    // get json property with name `property_name` from vertex with id `vid`
    async fn get_vertex_json_value(&self, vid: Uuid, property_name: &str) -> Result<Option<serde_json::Value>, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid.clone()).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Ok(None);
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() == 0 {
            return Ok(None);
        }

        Ok(Some(property_list.pop().unwrap().value))
    }

    // set json property with name `property_name` for vertex with id `vid`
    async fn set_vertex_json_value(&self, vid: Uuid, property_name: &str, json: &serde_json::Value) -> Result<bool, BackendError> {
        let trans = self.client.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Ok(false);
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        ct.async_set_vertex_properties(q, json).await?;
        Ok(true)
    }
}

#[derive(Clone)]
pub enum Request {
    Init(Vec<server::ServerInfo>),
    RegisterUser(String),
    CreateEmuNet(String, String, u32),
}

#[derive(Clone)]
pub enum Response {
    Init(QueryResult<()>),
    RegisterUser(QueryResult<()>),
    CreateEmuNet(QueryResult<Uuid>),
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

    // helper functions:
    async fn get_core_property<T: DeserializeOwned>(&self, property: &str) -> Result<T, BackendError> {
        let res = self.worker.get_vertex_json_value(CORE_INFO_ID.clone(), property).await?;
        match res {
            Some(jv) => Ok(serde_json::from_value(jv).unwrap()),
            None => panic!("database is not correctly initialized"),
        }
    }

    async fn set_core_property<T: Serialize>(&self, property: &str, t: T) -> Result<(), BackendError> {
        let jv = serde_json::to_value(t).unwrap();
        let res = self.worker.set_vertex_json_value(CORE_INFO_ID.clone(), property, &jv).await?;
        if !res {
            panic!("database is not correctly initialized");
        }
        Ok(())
    }
}

impl IndradbClientBackend {
    async fn init(&self, server_info_list: Vec<server::ServerInfo>) -> Result<QueryResult<()>, BackendError> {
        let res = self.worker.create_vertex(Some(CORE_INFO_ID.clone())).await?;
        match res {
            Some(_) => {
                // initialize user map
                self.set_core_property("user_map", HashMap::<String, user::EmuNetUser>::new()).await?;

                // initialize server list                
                self.set_core_property("server_info_list", server_info_list).await?;
                        
                Ok(QueryOk(()))
            },
            None => Ok(QueryFail("database has already been initialized".to_string())),
        }
    }

    async fn register_user(&self, user_id: String) -> Result<QueryResult<()>, BackendError> {        
        // read current user map
        let mut user_map: HashMap<String, user::EmuNetUser> = self.get_core_property("user_map").await?;
        if user_map.get(&user_id).is_some() {
            return Ok(QueryFail("user has already registered".to_string()));
        }

        // register the new user
        let user = user::EmuNetUser::new(&user_id);
        user_map.insert(user_id, user);        
        
        // sync update in the db
        self.set_core_property("user_map", user_map).await?;
        
        Ok(QueryOk(()))
    }

    async fn create_emu_net(&self, user: String, net: String, capacity: u32) -> Result<QueryResult<Uuid>, BackendError> {
        // get the user
        let mut user_map: HashMap<String, user::EmuNetUser> = self.get_core_property("user_map").await?;
        if user_map.get(&user).is_none() {
            return Ok(QueryFail("invalid user name".to_string()));
        }
        let user_mut = user_map.get_mut(&user).unwrap();

        // check whether the emunet has existed
        if user_mut.emu_net_exist(&net) {
            return Ok(QueryFail("invalid emu-net name".to_string()));
        }

        // get the allocation of servers
        let server_info_list: Vec<server::ServerInfo> = self.get_core_property("server_info_list").await?;
        let mut sp = server::ServerInfoList::from_iterator(server_info_list.into_iter()).unwrap();
        let allocation = match sp.allocate_servers(capacity) {
            Some(alloc) => alloc,
            None => return Ok(QueryFail("invalid capacity".to_string())),
        };
        self.set_core_property("server_info_list", sp.into_vec()).await?;


        // create a new emu net
        let mut emu_net = net::EmuNet::new(net.clone(), capacity);
        emu_net.add_servers(allocation);
        
        // create and initialize a new emu net node
        let emu_net_id = self.worker.create_vertex(None).await?.expect("vertex ID already exists");
        let jv = serde_json::to_value(emu_net).unwrap();
        let res = self.worker.set_vertex_json_value(emu_net_id, "default", &jv).await?;
        if !res {
            panic!("vertex not exist");
        }

        // add the new emunet to user map
        user_mut.add_emu_net(net, emu_net_id.clone());
        self.set_core_property("user_map", user_map).await?;

        Ok(QueryOk(emu_net_id))
    }
}

impl IndradbClientBackend {
    async fn dispatch_request(&self, req: Request) -> Result<Response, BackendError> {
        match req {
            Request::Init(server_infos) => {
                self.init(server_infos).await.map(|res|{Response::Init(res)})
            },
            Request::RegisterUser(user_name) => {
                self.register_user(user_name).await.map(|res|{Response::RegisterUser(res)})                
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
