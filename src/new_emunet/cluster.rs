use std::cmp::Ord;
use std::collections::HashSet;
use std::{cell::RefCell, collections::HashMap};

use serde::{Deserialize, Serialize};

use super::device::{DeviceInfo, LinkInfo};
use crate::algo::*;

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

    pub fn into_vec(self) -> Vec<ServerInfo> {
        self.servers
    }

    pub fn allocate_servers(&mut self, quantity: u64) -> Result<Vec<ContainerServer>, u64> {
        let mut target = 0;

        self.servers
            .sort_by(|a, b| (&b.max_capacity).cmp(&a.max_capacity));

        let mut index = 0;
        while target < quantity && index < self.servers.len() {
            target += self.servers[index].max_capacity;
            index += 1;
        }

        if target >= quantity {
            let res: Vec<_> = self
                .servers
                .drain(0..index)
                .map(|server_info| ContainerServer {
                    server_info,
                    devs: RefCell::new(HashSet::new()),
                })
                .collect();
            Ok(res)
        } else {
            Err(target)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerServer {
    server_info: ServerInfo,
    devs: RefCell<HashSet<u64>>,
}

impl ContainerServer {
    pub fn server_info(&self) -> &ServerInfo {
        return &self.server_info;
    }
}

impl PartitionBin for ContainerServer {
    type Item = u64;
    type BinId = uuid::Uuid;

    fn fill(&mut self, dev_id: Self::Item) -> bool {
        if self.devs.borrow().len() + 1 > self.server_info.max_capacity as usize {
            false
        } else {
            assert_eq!(self.devs.borrow_mut().insert(dev_id), true);
            true
        }
    }

    fn release(&mut self, dev_id: &Self::Item) -> bool {
        self.devs.borrow_mut().remove(dev_id)
    }

    fn bin_id(&self) -> Self::BinId {
        return self.server_info().uuid.clone();
    }
}

impl Max for u64 {
    fn maximum() -> Self {
        u64::MAX
    }
}

impl Min for u64 {
    fn minimum() -> Self {
        u64::MIN
    }
}

impl<'a, T, I> Partition<'a, ContainerServer, I>
    for UndirectedGraph<u64, DeviceInfo<T>, LinkInfo<T>>
where
    I: Iterator<Item = &'a mut ContainerServer>,
{
    type ItemId = u64;

    fn partition(
        &self,
        mut bins: I,
    ) -> Option<HashMap<Self::ItemId, <ContainerServer as PartitionBin>::BinId>> {  
        let mut dev_ids = self.nodes().map(|(nid, _)| *nid);

        let mut curr_server = bins.next()?;
        let mut res = HashMap::new();

        while let Some(dev_id) = dev_ids.next() {
            while !curr_server.fill(dev_id) {
                if let Some(new_server) = bins.next() {
                    curr_server = new_server;
                } else {
                    return None;
                }
            }
            res.insert(dev_id, curr_server.bin_id());
        }

        Some(res)
    }
}
