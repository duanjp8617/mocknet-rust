use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::cluster::ContainerServer;
use super::device::*;
use crate::algo::*;

pub(crate) static EMUNET_NODE_PROPERTY: &'static str = "default";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) enum EmunetError {
    PartitionFail(String),
    DatabaseFail(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) enum EmunetState {
    Uninit,
    Working,
    Normal,
    Error(EmunetError),
}

impl std::convert::From<EmunetState> for String {
    fn from(e: EmunetState) -> String {
        match e {
            EmunetState::Uninit => "Uninit".to_string(),
            EmunetState::Working => "Working".to_string(),
            EmunetState::Normal => "Normal".to_string(),
            EmunetState::Error(_) => "Error".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Emunet {
    emunet_name: String,
    emunet_uuid: uuid::Uuid,
    max_capacity: u64,
    user_name: String,
    state: RefCell<EmunetState>,
    dev_count: Cell<u64>,
    servers: RefCell<HashMap<uuid::Uuid, ContainerServer>>,
    devices: RefCell<HashMap<u64, Device<String, String>>>
}

impl Emunet {
    pub(crate) fn new(
        emunet_name: String,
        emunet_uuid: Uuid,
        user_name: String,
        servers: Vec<ContainerServer>,
    ) -> Self {
        let (hm, max_capacity) =
            servers
                .into_iter()
                .fold((HashMap::new(), 0), |(mut hm, mut max_capacity), cs| {
                    max_capacity += cs.server_info().max_capacity;
                    let cs_uuid = cs.server_info().uuid.clone();
                    hm.insert(cs_uuid, cs);
                    (hm, max_capacity)
                });

        Self {
            emunet_name,
            emunet_uuid,
            max_capacity,
            user_name,
            state: RefCell::new(EmunetState::Uninit),
            dev_count: Cell::new(0),
            servers: RefCell::new(hm),
            devices: RefCell::new(HashMap::new())
        }
    }
}

impl Emunet {
    pub(crate) fn max_capacity(&self) -> u64 {
        self.max_capacity
    }

    pub(crate) fn emunet_uuid(&self) -> Uuid {
        self.emunet_uuid.clone()
    }

    pub(crate) fn emunet_user(&self) -> String {
        self.user_name.clone()
    }

    pub(crate) fn emunet_name(&self) -> String {
        self.emunet_name.clone()
    }

    pub(crate) fn dev_count(&self) -> u64 {
        self.dev_count.get()
    }

    pub(crate) fn servers(&self) -> std::cell::Ref<HashMap<uuid::Uuid, ContainerServer>> {
        self.servers.borrow()
    }
}

impl Emunet {
    // modifying the state of the EmuNet
    pub(crate) fn state(&self) -> EmunetState {
        self.state.borrow().clone()
    }

    pub(crate) fn set_state(&self, state: EmunetState) {
        *self.state.borrow_mut() = state;
    }
}

impl Emunet {
    pub(crate) fn build_emunet_graph(
        &self,
        graph: &UndirectedGraph<u64, DeviceInfo<String>, LinkInfo<String>>,
    ) {
        assert_eq!(self.dev_count.get(), 0);
        assert_eq!(self.max_capacity >= graph.nodes_num() as u64, true);
        assert_eq!(self.devices.borrow().len(), 0);

        let mut servers_ref = self.servers.borrow_mut();
        let bins = servers_ref.values_mut();
        let assignment = graph
            .partition(bins)
            .expect("FATAL: this should always succeed");

        let total_devs = assignment.len();

        for (dev_id, server_uuid) in assignment.into_iter() {
            let dev_info = graph.get_node(dev_id).unwrap();
            let device = Device::new(dev_id, server_uuid, dev_info.meta().clone());

            let (out_edge_iter, in_edge_iter) = graph.edges_by_nid(dev_id);
            for (_, other) in out_edge_iter {
                let link_info = graph.get_edge((dev_id, other)).unwrap();
                let link = Link::new(dev_id, other, link_info.meta().clone());
                assert_eq!(device.add_link(link), true);
            }
            for (other, _) in in_edge_iter {
                let link_info = graph.get_edge((other, dev_id)).unwrap();
                let link = Link::new(dev_id, other, link_info.meta().clone());
                assert_eq!(device.add_link(link), true);
            }

            assert_eq!(
                self.devices.borrow_mut().insert(dev_id, device).is_none(),
                true
            );
        }

        self.dev_count.set(total_devs as u64);
    }

    pub(crate) fn release_emunet_graph(&self) -> UndirectedGraph<u64, String, String> {
        let mut nodes: Vec<(u64, String)> = Vec::new();
        let mut edges: Vec<((u64, u64), String)> = Vec::new();

        for (dev_id, dev) in self.devices.borrow().iter() {
            nodes.push((*dev_id, dev.meta().clone()));
            for link in dev.links().iter() {
                edges.push((link.link_id(), link.meta().clone()))
            }
        }

        UndirectedGraph::new(nodes, edges).unwrap()
    }

    pub(crate) fn release_emunet_resource(&self) -> Vec<ContainerServer> {
        let device_map = std::mem::replace(&mut *self.devices.borrow_mut(), HashMap::new());
        for (dev_id, dev) in device_map.into_iter() {
            self.servers
                .borrow_mut()
                .get_mut(&dev.server_uuid())
                .map(|cs| {
                    assert!(cs.release(&dev_id) == true);
                })
                .unwrap();
        }
        self.dev_count.set(0);
        let server_map = std::mem::replace(&mut *self.servers.borrow_mut(), HashMap::new());
        server_map.into_iter().map(|(_, cs)| cs).collect()
    }
}
