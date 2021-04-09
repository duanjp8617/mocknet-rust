use std::cmp::Ord;
use std::collections::HashSet;
use std::{cell::RefCell, collections::HashMap};

use serde::{Deserialize, Serialize};

use super::input_graph_format::{InputDevice, InputLink};
use crate::algo::*;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct EmunetAccessInfo {
    pub(crate) login_server_addr: String,
    pub(crate) login_server_user: String,
    pub(crate) login_server_pwd: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ServerInfo {
    pub(crate) node_name: String,
    pub(crate) max_capacity: u64,
}

#[derive(Deserialize)]
pub struct ClusterConfig {
    api_server_addr: String,
    access_info: EmunetAccessInfo,
    k8s_nodes: Vec<ServerInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ClusterInfo {
    api_server_addr: String,
    access_info: EmunetAccessInfo,
    servers: Vec<ServerInfo>,
}

impl ClusterInfo {
    fn node_name_exist(&self, node_name: &str) -> bool {
        let mut sorted: Vec<&str> = self.servers.iter().map(|e| &e.node_name[..]).collect();
        sorted.sort();
        sorted.binary_search(&node_name).is_ok()
    }

    fn add_server_info<S: std::convert::AsRef<str>>(
        &mut self,
        node_name: S,
        max_capacity: u64,
    ) -> Result<(), String> {
        if self.node_name_exist(node_name.as_ref()) {
            return Err(format!(
                "Address {:?} is already stored in the list.",
                node_name.as_ref()
            ));
        }

        self.servers.push(ServerInfo {
            node_name: node_name.as_ref().into(),
            max_capacity,
        });

        Ok(())
    }

    pub fn try_new(config: ClusterConfig) -> Result<Self, String> {
        let mut cluster_info = Self {
            api_server_addr: config.api_server_addr,
            access_info: config.access_info,
            servers: Vec::new(),
        };

        for node_info in config.k8s_nodes {
            cluster_info.add_server_info(node_info.node_name, node_info.max_capacity)?;
        }

        Ok(cluster_info)
    }

    pub(crate) fn rellocate_servers(
        &mut self,
        servers: Vec<ContainerServer>,
    ) -> Option<Vec<ContainerServer>> {
        for server in servers.iter() {
            assert!(server.devs().len() == 0);

            if self.node_name_exist(&server.server_info.node_name) {
                return Some(servers);
            }
        }
        for server in servers.into_iter() {
            self.servers.push(server.server_info)
        }
        None
    }

    pub(crate) fn into_vec(self) -> Vec<ServerInfo> {
        self.servers
    }

    pub(crate) fn allocate_servers(&mut self, quantity: u64) -> Result<Vec<ContainerServer>, u64> {
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

impl ClusterInfo {
    pub(crate) fn emunet_access_info(&self) -> &EmunetAccessInfo {
        &self.access_info
    }

    pub(crate) fn api_server_addr(&self) -> &str {
        &self.api_server_addr
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ContainerServer {
    server_info: ServerInfo,
    devs: RefCell<HashSet<u64>>,
}

impl ContainerServer {
    pub(crate) fn server_info(&self) -> &ServerInfo {
        return &self.server_info;
    }

    pub(crate) fn devs(&self) -> std::cell::Ref<HashSet<u64>> {
        self.devs.borrow()
    }
}

impl PartitionBin for ContainerServer {
    type Item = u64;
    type BinId = String;

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
        return self.server_info().node_name.clone();
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
    for UndirectedGraph<u64, InputDevice<T>, InputLink<T>>
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

#[derive(Serialize, Deserialize)]
pub(crate) struct IdAllocator {
    ids: HashSet<u8>,
}

impl IdAllocator {
    pub(crate) fn new() -> Self {
        let mut ids = HashSet::new();
        for i in 0..u8::MAX {
            ids.insert(i);
        }
        Self { ids }
    }

    pub(crate) fn alloc(&mut self) -> Option<u8> {
        let id_to_pop = self.ids.iter().next().map(|id| *id)?;
        assert!(self.ids.remove(&id_to_pop) == true);
        Some(id_to_pop)
    }

    pub(crate) fn realloc(&mut self, id: u8) -> bool {
        self.ids.insert(id)
    }

    pub(crate) fn remaining(&self) -> usize {
        self.ids.len()
    }
}
