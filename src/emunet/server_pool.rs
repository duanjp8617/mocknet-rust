use std::net::{IpAddr, SocketAddr};
use std::cmp::Ord;

use crate::util;

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct ServerAddr {
    conn_ip: IpAddr,
    conn_port: u16,
    data_ip: IpAddr,
    man_ip: IpAddr,
}

impl ServerAddr {
    fn new(conn_ip: String, conn_port: u16, data_ip: String, man_ip: String) -> Option<Self> {
        conn_ip.parse::<IpAddr>().ok().and_then(move |conn_ip| {
            data_ip.parse::<IpAddr>().ok().and_then(move |data_ip| {
                man_ip.parse::<IpAddr>().ok().map(move |man_ip|{
                    Self {
                        conn_ip,
                        conn_port,
                        data_ip,
                        man_ip
                    }
                })
            })
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ContainerServer {
    id: Uuid,
    server_addr: ServerAddr,
    capacity: u32,
}

impl ContainerServer {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn conn_addr(&self) -> SocketAddr {
        SocketAddr::new(self.server_addr.conn_ip, self.server_addr.conn_port)
    }

    pub fn data_ip(&self) -> IpAddr {
        self.server_addr.data_ip
    }

    pub fn man_ip(&self) -> IpAddr {
        self.server_addr.man_ip
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

pub struct ServerPool {
    servers: Vec<ContainerServer>
}

impl ServerPool {
    fn server_addr_exist(&self, server_addr: &ServerAddr) -> bool {
        let mut sorted: Vec<&ServerAddr> = self.servers.iter().map(|e|{&e.server_addr}).collect();
        sorted.sort();
        sorted.binary_search(&server_addr).is_ok()
    }

    pub fn new() -> Self {
        Self {
            servers: Vec::new()
        }
    }

    pub fn add_server(&mut self, conn_ip: &str, conn_port: u16, data_ip: &str, man_ip: &str, capacity: u32) {        
        let target = ServerAddr::new(conn_ip.to_string(), conn_port, data_ip.to_string(), man_ip.to_string()).expect("invalid server address");
        if self.server_addr_exist(&target) {
            panic!("ServerAddr {:?} exists in the pool", target);
        }
        
        self.servers.push(ContainerServer {
            id: util::new_uuid(),
            server_addr: target,
            capacity,
        });
    }

    pub fn add_servers<I>(&mut self, i: I) 
        where
            I: std::iter::Iterator<Item = ContainerServer>
    {
        for cs in i {
            if self.server_addr_exist(&cs.server_addr) {
                panic!("ServerAddr {:?} exists in the pool", &cs.server_addr);
            }
            self.servers.push(cs);
        }
    }

    pub fn into_vec(self) -> Vec<ContainerServer> {
        self.servers
    }

    pub fn allocate_servers(&mut self, quantity: u32) -> Option<Vec<ContainerServer>> {
        let mut res = Vec::new();
        let mut target = 0;
        
        while target < quantity && self.servers.len() > 0 {
            let cs = self.servers.pop().unwrap();
            target += cs.capacity();
            res.push(cs);
        };
        
        if target >= quantity {
            Some(res)
        }
        else {
            self.add_servers(res.into_iter());
            None
        }
    }
}