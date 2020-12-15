// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use futures::AsyncReadExt;
use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use uuid::Uuid;

use super::backend::IndradbClientBackend;
use super::backend::build_backend_fut;
use crate::emunet::{server, net};
use super::message::{Request, Response};
use super::message_queue;
use super::ClientError;
use super::errors::BackendError;
use super::QueryResult;
use super::request::{self, build_request};

/// The database client that stores core mocknet information.
pub struct Client {
    sender: message_queue::Sender<Request, Response, BackendError>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone()
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
        let req = request::Init::new(servers);
        let res = self.sender.send(build_request(req)).await?;
        match res {
            Response::Init(res) => Ok(res),
            _ => panic!("invalid response")
        }
    }

    /// Store a new user with `user_name`.
    /// 
    /// Return value has similar meaning as `Client::init`.
    pub async fn register_user(&self, user_name: &str) -> Result<QueryResult<()>, ClientError> {
        let req = request::RegisterUser::new(user_name.to_string());
        let res = self.sender.send(build_request(req)).await?;
        match res {
            Response::RegisterUser(res) => Ok(res),
            _ => panic!("invalid response")
        }
    }

    /// Create a new emulation net for `user` with `name` and `capacity`.
    /// 
    /// Return value has similar meaning as `Client::init`.
    pub async fn create_emu_net(&self, user: String, net: String, capacity: u32) -> Result<QueryResult<Uuid>, ClientError> {
        let req= request::CreateEmuNet::new(user.to_string(), net.to_string(), capacity);
        let res = self.sender.send(build_request(req)).await?;
        match res {
            Response::CreateEmuNet(res) => Ok(res),
            _ => panic!("invalid response")
        }
    }

    /// List all the emunet of a user.
    /// 
    /// Note: I don't know if this is necessary
    pub async fn list_emu_net_uuid(&self, user: String) -> Result<QueryResult<HashMap<String, Uuid>>, ClientError> {
        let req = request::ListEmuNet::new(user);
        let res = self.sender.send(build_request(req)).await?;
        match res {
            Response::ListEmuNet(res) => Ok(res),
            _ => panic!("invalid response")
        }
    }

    /// Get the emunet from an uuid.
    /// 
    /// Note: I don't know if this is necessary as well.
    pub async fn get_emu_net(&self, uuid: Uuid) -> Result<QueryResult<net::EmuNet>, ClientError> {
        let req = request::GetEmuNet::new(uuid);
        let res = self.sender.send(build_request(req)).await?;
        match res {
            Response::GetEmuNet(res) => Ok(res),
            _ => panic!("invalid response")
        } 
    }

    /// Get the emunet from an uuid.
    /// 
    /// Note: I don't know if this is necessary as well.
    pub async fn set_emu_net(&self, emu_net: net::EmuNet) -> Result<QueryResult<()>, ClientError> {
        let req = request::SetEmuNet::new(emu_net);
        let res = self.sender.send(build_request(req)).await?;
        match res {
            Response::SetEmuNet(res) => Ok(res),
            _ => panic!("invalid response")
        } 
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
            let indradb_client_backend = IndradbClientBackend::new(indradb_capnp_client, disconnector);
    
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
        let client = Client{sender};
        let entry_fn_jh = tokio::spawn(entry_fn(client));

        backend_fut.await?;
        entry_fn_jh.await.unwrap()
    }
}