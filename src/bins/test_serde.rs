use mocknet::util;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use std::net::{SocketAddr, ToSocketAddrs, IpAddr};

use mocknet::emunet::server;

#[derive(Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
    uuid: Uuid,
    addr: SocketAddr,
    ip: IpAddr,
}

#[allow(dead_code)]
fn print_an_address() -> Result<()> {
    // Some data structure.
    let address = Address {
        street: "10 Downing Street".to_owned(),
        city: "London".to_owned(),
        uuid: util::new_uuid(),
        addr: "127.0.0.1:5123".to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address"),
        ip: "192.167.2.5".parse::<IpAddr>().unwrap(),
    };

    // Serialize it to a JSON string.
    let j = serde_json::to_string(&address)?;

    // Print, write to a file, or send to an HTTP server.
    println!("{}", j);

    Ok(())
}

#[allow(dead_code)]
fn test_server_pool() {
    let mut sp = server::ServerPool::new();
    sp.add_server("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5);
    sp.add_server("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7);
    // sp.add_server("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 10);
    // sp.add_server("127.0.0.2.5", 128, "128.0.0.3", "129.0.0.4", 10);

    let ls = sp.into_vec();
    for cs in ls.iter() {
        println!("{}", cs.conn_addr());
    }

    let mut sp = server::ServerPool::new();
    sp.add_server("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9);
    sp.add_server("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10);
    // sp.add_server("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 10);
    // sp.add_server("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 10);
    sp.add_servers(ls.into_iter());

    let ls = sp.allocate_servers(19).unwrap();
    println!("first 19: {}", ls.len());
    for i in ls.iter() {
        println!("first 19: {}", i.capacity());
    }

    let ls = sp.allocate_servers(19);
    println!("second none: {}", ls.is_none());

    let ls = sp.allocate_servers(10).unwrap();
    println!("last 10: {}", ls.len());
    for i in ls.iter() {
        println!("last 10: {}", i.capacity());
    }
}

fn main() {
    // print_an_address().unwrap();
    test_server_pool();
}