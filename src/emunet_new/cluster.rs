use std::{cell::Cell, cmp::Ord};

use serde::{Deserialize, Serialize};

use crate::algo::PartitionBin;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerInfo {
    id: uuid::Uuid,
    conn_addr: std::net::IpAddr,
    max_capacity: u64,
    username: String,
    password: String,
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

    fn addr_exist(&self, server_addr: &std::net::IpAddr) -> Result<(), ()> {
        let mut sorted: Vec<&std::net::IpAddr> =
            self.servers.iter().map(|e| &e.conn_addr).collect();
        sorted.sort();
        sorted
            .binary_search(&server_addr)
            .map(|_| {})
            .map_err(|_| {})
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

        self.addr_exist(&conn_addr)
            .map_err(|_| format!("Address {:?} is already stored in the list.", &conn_addr))?;

        self.servers.push(ServerInfo {
            id: indradb::util::generate_uuid_v1(),
            conn_addr,
            max_capacity,
            username: username.into(),
            password: password.into(),
        });

        Ok(())
    }

    pub fn from_iterator<I: std::iter::Iterator<Item = ServerInfo>>(i: I) -> Result<Self, String> {
        let mut res = Self::new();
        for cs in i {
            res.addr_exist(&cs.conn_addr)
                .map_err(|_| format!("ServerAddr {:?} exists in the pool", &cs.conn_addr))?;
            res.servers.push(cs);
        }
        Ok(res)
    }

    pub fn into_vec(self) -> Vec<ServerInfo> {
        self.servers
    }

    pub fn allocate_servers(&mut self, quantity: u64) -> Result<Vec<ContainerServer>, u64> {
        let mut target = 0;

        let mut enumerate: Vec<(usize, u64)> = self
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
                        dev_count: Cell::new(0),
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
    dev_count: Cell<u64>,
}

// impl ContainerServer {
//     pub fn get_server_info(self) -> ServerInfo {
//         return self.server_info;
//     }
// }

impl ContainerServer {
    pub fn id(&self) -> uuid::Uuid {
        return self.server_info.id;
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
        return self.id();
    }
}
