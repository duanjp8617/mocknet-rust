use std::{cell::Cell, cmp::Ord};

use serde::{Deserialize, Serialize};

use crate::algo::PartitionBin;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerInfo {
    pub uuid: uuid::Uuid,
    pub conn_addr: std::net::IpAddr,
    pub max_capacity: u64,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClusterInfo {
    servers: Vec<ServerInfo>,
}

impl ClusterInfo {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    fn addr_exist(&self, server_addr: &std::net::IpAddr) -> bool {
        let mut sorted: Vec<&std::net::IpAddr> =
            self.servers.iter().map(|e| &e.conn_addr).collect();
        sorted.sort();
        sorted.binary_search(&server_addr).is_ok()
    }

    pub fn add_server_info<S: std::convert::Into<String>>(
        &mut self,
        conn_ip: S,
        max_capacity: u64,
        username: S,
        password: S,
    ) -> Result<(), String> {
        let conn_addr = conn_ip
            .into()
            .parse::<std::net::IpAddr>()
            .map_err(|e| format!("{:?}", e))?;

        if self.addr_exist(&conn_addr) {
            return Err(format!(
                "Address {:?} is already stored in the list.",
                &conn_addr
            ));
        }

        self.servers.push(ServerInfo {
            uuid: indradb::util::generate_uuid_v1(),
            conn_addr,
            max_capacity,
            username: username.into(),
            password: password.into(),
        });

        Ok(())
    }

    // pub fn from_iterator<I: std::iter::Iterator<Item = ServerInfo>>(i: I) -> Result<Self, String> {
    //     let mut res = Self::new();
    //     for cs in i {
    //         if res.addr_exist(&cs.conn_addr) {
    //             return Err(format!("ServerAddr {:?} exists in the pool", &cs.conn_addr));
    //         }
    //         res.servers.push(cs);
    //     }
    //     Ok(res)
    // }

    // pub fn into_vec(self) -> Vec<ServerInfo> {
    //     self.servers
    // }

    pub fn allocate_servers(&mut self, quantity: u64) -> Result<Vec<ContainerServer>, u64> {
        let mut target = 0;

        self.servers.sort_by(|a, b| (&b.max_capacity).cmp(&a.max_capacity));

        let mut index = 0;
        while target < quantity && index < self.servers.len() {
            target += self.servers[index].max_capacity;
            index += 1;
        }

        if target >= quantity {
            let res: Vec<_> = self.servers.drain(0..index).map(|server_info| {
                ContainerServer {
                    server_info,
                    dev_count: Cell::new(0),
                }
            }).collect();
            Ok(res)
        } else {
            Err(target)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerServer {
    server_info: ServerInfo,
    dev_count: Cell<u64>,
}

// impl ContainerServer {
//     pub fn get_server_info(self) -> ServerInfo {
//         return self.server_info;
//     }
// }

impl ContainerServer {
    pub fn server_info(&self) -> &ServerInfo {
        return &self.server_info;
    }

    // pub fn release_resource(&mut self, quantity: usize) -> Result<(), ()> {
    //     if self.curr_capacity + quantity <= self.server_info.max_capacity {
    //         self.curr_capacity += quantity;
    //         Ok(())
    //     } else {
    //         Err(())
    //     }
    // }
}

impl PartitionBin for ContainerServer {
    type Size = u64;
    type BinId = uuid::Uuid;

    fn fill(&mut self, dev_num: Self::Size) -> bool {
        if self.dev_count.get() + dev_num > self.server_info.max_capacity {
            return false;
        } else {
            self.dev_count.set(self.dev_count.get() + dev_num);
            return true;
        }
    }

    fn release(&mut self, dev_num: Self::Size) -> bool {
        if self.dev_count.get() < dev_num {
            return false;
        } else {
            self.dev_count.set(self.dev_count.get() - dev_num);
            return true;
        }
    }

    fn bin_id(&self) -> Self::BinId {
        return self.server_info().uuid.clone();
    }
}
