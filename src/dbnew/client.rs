// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use futures::AsyncReadExt;
use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use uuid::Uuid;
use indradb::{BulkInsertItem, Vertex, EdgeKey, Type};

use super::indradb::Backend as IndradbBackend;
use super::indradb::build_backend_fut;
use super::indradb::message_queue;
// use super::backend::build_backend_fut;
use crate::emunet::{server, user, net};
// use super::message::{Request, Response};
// use super::message_queue;
use super::ClientError;
use super::errors::BackendError;
// use super::request::{self};
use super::indradb::Frontend as IndradbFrontend;
use super::CORE_INFO_ID;

type QueryResult<T> = Result<T, String>;

macro_rules! succeed {
    ($arg: expr) => {
         Ok(Ok($arg))
     }
}

macro_rules! fail {
    ($s: expr) => {
        Ok(Err($s))
    }
}


/// The database client that stores core mocknet information.
pub struct Client {
    fe: IndradbFrontend,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            fe: self.fe.clone()
        }
    }
}

impl Client {
    /// Initilize a table for storing core information of the mocknet database.
    /// 
    /// `servers` stores information about backend servers for launching containers.
    /// 
    /// Interpretation of return values:
    /// Ok(Ok(())) means successful initialization.
    /// Ok(Err(s)) means the database has been initialized, and `s` is the error message.
    /// Err(e) means fatal errors occur, the errors include disconnection with backend servers and 
    /// dropping backend worker (though the second error si unlikely to occur.)
    pub async fn init(&self, servers: Vec<server::ServerInfo>) -> Result<QueryResult<()>, ClientError> {
        let res = self.fe.create_vertex(Some(CORE_INFO_ID.clone())).await?;
        match res {
            Some(_) => {
                // initialize user map
                self.fe.set_user_map(HashMap::<String, user::EmuNetUser>::new()).await?;

                // initialize server list                
                self.fe.set_server_info_list(servers).await?;
                        
                succeed!(())
            },
            None => fail!("database has already been initialized".to_string()),
        }
    }

    /// Store a new user with `user_name`.
    /// 
    /// Return value has similar meaning as `Client::init`.
    pub async fn register_user(&self, user_name: &str) -> Result<QueryResult<()>, ClientError> {
        // read current user map
        let mut user_map: HashMap<String, user::EmuNetUser> = self.fe.get_user_map().await?;
        if user_map.get(user_name).is_some() {
            return fail!("user has already registered".to_string());
        }

        // register the new user
        let user = user::EmuNetUser::new(&user_name);
        user_map.insert(user_name.to_string(), user);
        
        // sync update in the db
        self.fe.set_user_map(user_map).await?;
        
        succeed!(())
    }


    /// Create a new emulation net for `user` with `name` and `capacity`.
    /// 
    /// Return value has similar meaning as `Client::init`.
    pub async fn create_emu_net(&self, user: String, net: String, capacity: u32) -> Result<QueryResult<Uuid>, ClientError> {
        // get the user
        let mut user_map: HashMap<String, user::EmuNetUser> = self.fe.get_user_map().await?;
        if user_map.get(&user).is_none() {
            return fail!("invalid user name".to_string());
        }
        let user_mut = user_map.get_mut(&user).unwrap();

        // check whether the emunet has existed
        if user_mut.emu_net_exist(&net) {
            return fail!("invalid emu-net name".to_string())
        }

        // get the allocation of servers
        let server_info_list: Vec<server::ServerInfo> = self.fe.get_server_info_list().await?;
        let mut sp = server::ServerInfoList::from_iterator(server_info_list.into_iter()).unwrap();
        let allocation = match sp.allocate_servers(capacity) {
            Some(alloc) => alloc,
            None => return fail!("invalid capacity".to_string()),
        };
        self.fe.set_server_info_list(sp.into_vec()).await?;
        
        // create a new emu net node
        let emu_net_id = self.fe.create_vertex(None).await?.expect("vertex ID already exists");
        // create a new emu net
        let mut emu_net = net::EmuNet::new(user, net.clone(), emu_net_id.clone(), capacity);
        emu_net.add_servers(allocation);
        // initialize the EmuNet in the database
        let jv = serde_json::to_value(emu_net).unwrap();
        let res = self.fe.set_vertex_json_value(emu_net_id, "default", jv).await?;
        if !res {
            panic!("vertex not exist");
        }

        // add the new emunet to user map
        user_mut.add_emu_net(net, emu_net_id.clone());
        self.fe.set_user_map(user_map).await?;

        succeed!(emu_net_id)
    }

    /// List all the emunet of a user.
    /// 
    /// Note: I don't know if this is necessary
    pub async fn list_emu_net_uuid(&self, user: String) -> Result<QueryResult<HashMap<String, Uuid>>, ClientError> {
        // get user
        let user_map: HashMap<String, user::EmuNetUser> = self.fe.get_user_map().await?;
        if !user_map.contains_key(&user) {
            return fail!("invalid user name".to_string());
        }
        let user = user_map.get(&user).unwrap();
        
        succeed!(user.get_all_emu_nets())
    }

    /// Get the emunet from an uuid.
    /// 
    /// Note: I don't know if this is necessary as well.
    pub async fn get_emu_net(&self, uuid: Uuid) -> Result<QueryResult<net::EmuNet>, ClientError> {
        let res = self.fe.get_vertex_json_value(uuid, "default").await?;
        match res {
            None => fail!("emunet not exist".to_string()),
            Some(jv) => succeed!(serde_json::from_value(jv).unwrap()),
        }
    }

    /// Get the emunet from an uuid.
    /// 
    /// Note: I don't know if this is necessary as well.
    pub async fn set_emu_net(&self, emu_net: net::EmuNet) -> Result<QueryResult<()>, ClientError> {
        let uuid = emu_net.uuid().clone();
        let jv = serde_json::to_value(emu_net).unwrap();
        let res = self.fe.set_vertex_json_value(uuid, "default", jv).await?;
        match res {
            false => fail!("EmuNet not exist".to_string()),
            true => succeed!(()),
        }
    }

    /// Create a bulk of vertexes from a vector of vertex uuids.
    /// 
    /// Note, we assume this method to be never fail. 
    /// However, if there is an uuid collision, this method can still finish without 
    /// returning useful error messages. 
    /// Consider repairing this in the future?
    pub async fn bulk_create_vertexes(&self, vertexes: Vec<Uuid>, t: String) -> Result<QueryResult<()>, ClientError> {
        let qs: Vec<BulkInsertItem> = vertexes.into_iter().fold(Vec::new(), |mut qs, uuid| {
            let v = Vertex::with_id(uuid, Type::new(&t).unwrap());
            qs.push(BulkInsertItem::Vertex(v));
            qs
        });

        self.fe.bulk_insert(qs).await?;
        succeed!(())
    }

    /// Set properties for all the vertexes from the list.
    /// 
    /// Note, we assume this method to be never fail. 
    /// However, if a particular vertex is not created in the datbase, this method can still finish without 
    /// returning useful error messages. 
    /// Consider repairing this in the future?
    pub async fn bulk_set_vertex_properties(&self, vertex_properties: Vec<(Uuid, serde_json::Value)>) 
    -> Result<QueryResult<()>, ClientError> 
    {
        let qs: Vec<BulkInsertItem> = vertex_properties.into_iter().fold(Vec::new(), |mut qs, vertex_property| {            
            qs.push(BulkInsertItem::VertexProperty(vertex_property.0, "default".to_string(), vertex_property.1));
            qs
        });

        self.fe.bulk_insert(qs).await?;
        succeed!(())
    }
}

/// The launcher that runs the client in a closure. 
pub struct ClientLauncher {
    conn: tokio::net::TcpStream,
}

impl ClientLauncher {
    /// Make an async connection to the database and return a ClientLauncher.
    pub async fn connect(addr: &std::net::SocketAddr) -> Result<Self, std::io::Error> {
        let conn = tokio::net::TcpStream::connect(&addr).await?;
        Ok(Self {conn})
    }

    /// Launch a background task and run the entry function.
    /// 
    /// The entry function is the start point of the mocknet program.
    pub async fn with_db_client<Func, Fut>(self, entry_fn: Func) -> Result<(), Box<dyn std::error::Error + Send>> 
        where
            Func: Fn(Client) -> Fut,
            Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + 'static + Send, 
    {
        let ls = tokio::task::LocalSet::new();
        let (sender, queue) = message_queue::create();
        
        // every capnp-related struct is non Send, so must be launched in LocalSet
        let backend_fut = ls.run_until(async move {         
            // create rpc_system
            let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(self.conn).split();
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
            let indradb_client_backend = IndradbBackend::new(indradb_capnp_client, disconnector);
    
            // run rpc_system
            tokio::task::spawn_local(async move {
                capnp_rpc_system.await
            });

            // run indradb backend
            tokio::task::spawn_local(build_backend_fut(indradb_client_backend, queue))
                .await
                .unwrap()
        });

        // launch the backend task to run entry function
        let client = Client {
            fe: IndradbFrontend::new(sender)
        };
        let entry_fn_jh = tokio::spawn(entry_fn(client));

        backend_fut.await?;
        entry_fn_jh.await.unwrap()
    }
}