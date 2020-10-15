
// use mocknet::backend::conn_service;
use mocknet::autogen;
use indradb;

use std::net::ToSocketAddrs;

use futures::AsyncReadExt;
use futures::FutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "192.168.109.57:10240"
    .to_socket_addrs()
    .unwrap()
    .next()
    .expect("could not parse address");
    
    let stream = tokio::net::TcpStream::connect(&addr).await?;
    stream.set_nodelay(true)?;
    let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();
    println!("The connection is successful");
    Ok(())
}