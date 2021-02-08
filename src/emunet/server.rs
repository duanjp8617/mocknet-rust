use std::cmp::Ord;
use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::algo::PartitionBin;

// The IP addresses that are used to talk to the server.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct ServerAddress {
    conn_ip: IpAddr,
    conn_port: u16,
    data_ip: IpAddr,
    man_ip: IpAddr,
}

impl ServerAddress {
    fn new(conn_ip: &str, conn_port: u16, data_ip: &str, man_ip: &str) -> Option<Self> {
        conn_ip.parse::<IpAddr>().ok().and_then(move |conn_ip| {
            data_ip.parse::<IpAddr>().ok().and_then(move |data_ip| {
                man_ip.parse::<IpAddr>().ok().map(move |man_ip| Self {
                    conn_ip,
                    conn_port,
                    data_ip,
                    man_ip,
                })
            })
        })
    }
}

/// Core information of a server.
///
/// `id`: the id of the server,
/// `server_addr`: server addresses,
/// `max_capacity`: the maximum number of containers that can be launched in the server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerInfo {
    id: Uuid,
    server_addr: ServerAddress,
    max_capacity: u32,
}

/// A list of `ServerInfo` that can be stored in the database as JSON value.
#[derive(Serialize, Deserialize)]
pub struct ServerInfoList {
    servers: Vec<ServerInfo>,
}

impl ServerInfoList {
    fn server_addr_exist(&self, server_addr: &ServerAddress) -> bool {
        let mut sorted: Vec<&ServerAddress> = self.servers.iter().map(|e| &e.server_addr).collect();
        sorted.sort();
        sorted.binary_search(&server_addr).is_ok()
    }

    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// Add a new server to the list.
    pub fn add_server_info(
        &mut self,
        conn_ip: &str,
        conn_port: u16,
        data_ip: &str,
        man_ip: &str,
        max_capacity: u32,
    ) -> Result<(), String> {
        let address = ServerAddress::new(conn_ip, conn_port, data_ip, man_ip).unwrap();
        // validate the address
        if self.server_addr_exist(&address) {
            return Err(format!(
                "Address {:?} is already stored in the list.",
                &address
            ));
        }

        self.servers.push(ServerInfo {
            id: indradb::util::generate_uuid_v1(),
            server_addr: address,
            max_capacity,
        });

        Ok(())
    }

    /// Build `Self` from `Iterator`.
    pub fn from_iterator<I: std::iter::Iterator<Item = ServerInfo>>(i: I) -> Result<Self, String> {
        let mut res = Self::new();
        for cs in i {
            if res.server_addr_exist(&cs.server_addr) {
                return Err(format!(
                    "ServerAddr {:?} exists in the pool",
                    &cs.server_addr
                ));
            }
            res.servers.push(cs);
        }
        Ok(res)
    }

    /// Convert `Self` into a `Vec`.
    pub fn into_vec(self) -> Vec<ServerInfo> {
        self.servers
    }

    /// Use a simple greedy algorithm to allocate servers
    pub fn allocate_servers(&mut self, quantity: u32) -> Result<Vec<ContainerServer>, u32> {
        let mut target = 0;

        let mut enumerate: Vec<(usize, u32)> = self
            .servers
            .iter()
            .map(|e| e.max_capacity)
            .enumerate()
            .collect();
        enumerate.sort_by(|a, b| (&b.1).cmp(&a.1));

        let mut index = 0;
        while target < quantity && index < enumerate.len() {
            target += enumerate[index].1;
            index += 1;
        }

        if target >= quantity {
            Ok(enumerate
                .iter()
                .take(index)
                .map(|e| {
                    let server_info = self.servers.remove(e.0);
                    let curr_capacity = server_info.max_capacity;
                    ContainerServer {
                        server_info,
                        curr_capacity,
                    }
                })
                .collect())
        } else {
            Err(target)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerServer {
    server_info: ServerInfo,
    curr_capacity: u32,
}

impl ContainerServer {
    pub fn id(&self) -> Uuid {
        return self.server_info.id;
    }

    pub fn conn_addr(&self) -> SocketAddr {
        let server_addr = &self.server_info.server_addr;
        SocketAddr::new(server_addr.conn_ip, server_addr.conn_port)
    }

    pub fn release_resource(&mut self, quantity: u32) -> Result<(), ()> {
        if self.curr_capacity + quantity <= self.server_info.max_capacity {
            self.curr_capacity += quantity;
            Ok(())
        } else {
            Err(())
        }
    }
}

impl PartitionBin for ContainerServer {
    type Size = u32;
    type BinId = Uuid;

    fn fill(&mut self, resource_size: u32) -> bool {
        if self.curr_capacity < resource_size {
            return false;
        } else {
            self.curr_capacity -= resource_size;
            return true;
        }
    }

    fn bin_id(&self) -> Self::BinId {
        return self.id();
    }
}
