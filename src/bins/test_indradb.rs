
// use mocknet::backend::conn_service;
use mocknet::autogen;
use indradb;

use std::net::ToSocketAddrs;
use std::future::Future;

use futures::AsyncReadExt;
use futures::FutureExt;

use capnp::Error as CapnpError;
use capnp_rpc::rpc_twoparty_capnp::Side;
use capnp_rpc::{twoparty, RpcSystem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "192.168.109.57:10240"
    .to_socket_addrs()
    .unwrap()
    .next()
    .expect("could not parse address");
    
    tokio::task::LocalSet::new().run_until(async move {
        let stream = tokio::net::TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
        
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            Side::Client,
            Default::default(),
        ));
        let mut rpc_system = RpcSystem::new(rpc_network, None);
        let client: autogen::service::Client = rpc_system.bootstrap(Side::Server);

        tokio::task::spawn_local(rpc_system.map(|_| ()));

        let req = client.ping_request();
        let res = req.send().promise.await?;

        if res.get().unwrap().get_ready() {
            println!("yeah");
            Ok(())
        } else {
            println!("no");
            Ok(())
        }

    }).await
}