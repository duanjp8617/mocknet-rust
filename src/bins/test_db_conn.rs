
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

use mocknet::storage::db_conn::{DBConn, DBReqSender, DBConnLoop};
use mocknet::errors::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:27615"
    .to_socket_addrs()
    .unwrap()
    .next()
    .expect("could not parse address");
    
    let db_conn = DBConn::new(&addr).await?;
    println!("connection successful");
    let (mut req_sender, conn_loop) = db_conn.launch();
    let ls = tokio::task::LocalSet::new();

    let res = ls.run_until(async move {
        conn_loop.await
    });

    // tokio::spawn(async move {
        req_sender.ping();
    // });

    let _  = res.await?;
    Ok(())
}