
// use mocknet::backend::conn_service;
use mocknet::autogen;
// use indradb;

use std::net::ToSocketAddrs;
// use std::future::Future;

use futures::AsyncReadExt;
use futures::FutureExt;

// use capnp::Error as CapnpError;
use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:27615"
    .to_socket_addrs()
    .unwrap()
    .next()
    .expect("could not parse address");
    let ls = tokio::task::LocalSet::new();
    let jh = ls.run_until(async move {
        println!("running");
        let stream = tokio::net::TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
        
        // create an rpc_network
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            Side::Client,
            Default::default(),
        ));
        // create a rpc_system from the rpc_network.
        // Note: the rpc_system drives the underlying connection, so 
        // we must spawn it in a task
        let mut rpc_system = RpcSystem::new(rpc_network, None);
        // create a new client from the rpc_system
        let client: autogen::service::Client = rpc_system.bootstrap(Side::Server);
        // spawn the rpc_system in a taks to drive the underlying network connection
        tokio::task::spawn_local(rpc_system.map(|_| ()));

        tokio::spawn(async move {});
        // The following code generates a ping request to the indradb
        // to verify whether the indradb works normally.
        let req = client.ping_request();
        let res = req.send().promise.await?;
        if res.get().unwrap().get_ready() {
            println!("yeah");
        } else {
            println!("no");
        }

        // create a new transaction from the client
        let _ = client.transaction_request().send().pipeline.get_transaction();



        Ok(())

    });
    jh.await
}