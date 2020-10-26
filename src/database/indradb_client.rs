// An implementation of Indradb storage backend
use std::future::Future;

use futures::AsyncReadExt;

use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};
use capnp_rpc::Disconnector;

use crate::emunet::server;
use super::message_queue::{Sender, create};

use super::indradb_backend::{Request, Response, IndradbClientBackend};
use super::indradb_backend::build_backend_fut;

pub type IndradbClientError = super::message_queue::error::MsgQError<capnp::Error>;

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

    pub async fn register_user(&self, user_name: String) -> Result<bool, IndradbClientError> {
        let req = Request::RegisterUser(user_name);
        let res = self.sender.send(req).await?;
        match res {
            Response::RegisterUser(res) => Ok(res),
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
        let indradb_client_backend = IndradbClientBackend::new(indradb_capnp_client, disconnector);

        // run rpc_system
        tokio::task::spawn_local(async move {
            capnp_rpc_system.await
        });
        // run indradb backend
        tokio::task::spawn_local(build_backend_fut(indradb_client_backend, queue)).await.unwrap()
    });
    
    (IndradbClient{sender}, backend_fut)
}