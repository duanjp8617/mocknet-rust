// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use futures::AsyncReadExt;
use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use uuid::Uuid;

use super::indradb_backend::{Request, Response, IndradbClientBackend};
use super::indradb_backend::build_backend_fut;
use crate::emunet::{server, net};
use super::message_queue;
use super::ClientError;
use super::errors::BackendError;
use super::QueryResult;

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