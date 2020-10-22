use mocknet::util;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use std::net::{SocketAddr, ToSocketAddrs, IpAddr};

#[derive(Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
    uuid: Uuid,
    addr: SocketAddr,
    ip: IpAddr,
}

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

fn main() {
    print_an_address().unwrap();
}