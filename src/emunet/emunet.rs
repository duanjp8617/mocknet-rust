use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::cluster::{ContainerServer, EmunetAccessInfo};
use super::device::*;
use super::device_metadata::*;
use super::utils::Ipv4AddrAllocator;
use crate::algo::*;
use crate::k8s_api::{EmunetReq, Topology, TopologyLinks, TopologyMeta};

pub(crate) static EMUNET_NODE_PROPERTY: &'static str = "default";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) enum EmunetState {
    Uninit,
    Working,
    Normal,
    Error(String),
}

impl std::convert::From<EmunetState> for String {
    fn from(e: EmunetState) -> String {
        match e {
            EmunetState::Uninit => "uninit".to_string(),
            EmunetState::Working => "working".to_string(),
            EmunetState::Normal => "normal".to_string(),
            EmunetState::Error(inner) => format!("error: {}", inner),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Emunet {
    emunet_name: String,
    emunet_uuid: uuid::Uuid,
    max_capacity: u64,
    user_name: String,
    api_server_addr: String,
    access_info: EmunetAccessInfo,
    state: RefCell<EmunetState>,
    dev_count: Cell<u64>,
    servers: RefCell<HashMap<String, ContainerServer>>,
    devices: RefCell<HashMap<u64, Device<DeviceMeta, LinkMeta>>>,
    addr_allocator: RefCell<Ipv4AddrAllocator>,
}

impl Emunet {
    pub(crate) fn new(
        emunet_name: String,
        emunet_uuid: Uuid,
        user_name: String,
        api_server_addr: String,
        access_info: EmunetAccessInfo,
        servers: Vec<ContainerServer>,
    ) -> Self {
        let (hm, max_capacity) =
            servers
                .into_iter()
                .fold((HashMap::new(), 0), |(mut hm, mut max_capacity), cs| {
                    max_capacity += cs.server_info().max_capacity;
                    let cs_name = cs.server_info().node_name.clone();
                    hm.insert(cs_name, cs);
                    (hm, max_capacity)
                });

        Self {
            emunet_name,
            emunet_uuid,
            max_capacity,
            user_name,
            api_server_addr,
            access_info,
            state: RefCell::new(EmunetState::Uninit),
            dev_count: Cell::new(0),
            servers: RefCell::new(hm),
            devices: RefCell::new(HashMap::new()),
            addr_allocator: RefCell::new(Ipv4AddrAllocator::new()),
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

    pub(crate) fn servers(&self) -> std::cell::Ref<HashMap<String, ContainerServer>> {
        self.servers.borrow()
    }

    pub(crate) fn api_server_addr(&self) -> String {
        self.api_server_addr.clone()
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
        assert_eq!(
            self.addr_allocator.borrow().remaining_addrs() >= graph.edges_num() * 2,
            true
        );

        let mut servers_ref = self.servers.borrow_mut();
        let bins = servers_ref.values_mut();
        let assignment = graph
            .partition(bins)
            .expect("FATAL: this should always succeed");

        let total_devs = assignment.len();

        for (dev_id, server_name) in assignment.into_iter() {
            let device = Device::new(
                dev_id,
                server_name.clone(),
                DeviceMeta::new(dev_id, &self.emunet_name, &server_name),
            );

            let (out_edge_iter, in_edge_iter) = graph.edges_by_nid(dev_id);
            for (_, other) in out_edge_iter {
                let link_meta = LinkMeta::new(
                    dev_id,
                    other,
                    device.meta().generate_intf_name(),
                    self.addr_allocator.borrow_mut().try_alloc().unwrap(),
                );
                let link = Link::new(dev_id, other, link_meta);
                assert_eq!(device.add_link(link), true);
            }
            for (other, _) in in_edge_iter {
                let link_meta = LinkMeta::new(
                    dev_id,
                    other,
                    device.meta().generate_intf_name(),
                    self.addr_allocator.borrow_mut().try_alloc().unwrap(),
                );
                let link = Link::new(dev_id, other, link_meta);
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
            nodes.push((*dev_id, serde_json::to_string(dev.meta()).unwrap()));
            for link in dev.links().iter() {
                edges.push((link.link_id(), serde_json::to_string(link.meta()).unwrap()))
            }
        }

        UndirectedGraph::new(nodes, edges).unwrap()
    }

    pub(crate) fn release_emunet_resource(&self) -> Vec<ContainerServer> {
        let device_map = std::mem::replace(&mut *self.devices.borrow_mut(), HashMap::new());
        for (dev_id, dev) in device_map.into_iter() {
            self.servers
                .borrow_mut()
                .get_mut(&dev.server_name())
                .map(|cs| {
                    assert!(cs.release(&dev_id) == true);
                })
                .unwrap();
        }
        self.dev_count.set(0);
        let server_map = std::mem::replace(&mut *self.servers.borrow_mut(), HashMap::new());
        server_map.into_iter().map(|(_, cs)| cs).collect()
    }

    pub(crate) fn release_grpc_messages(&self) -> EmunetReq {
        let mut pods = Vec::new();
        let mut topologies = Vec::new();

        for (_, dev) in self.devices.borrow().iter() {
            pods.push(dev.meta().get_pod());

            let mut topology_links = Vec::new();
            for link in dev.links().iter() {
                let link_id = link.link_id();
                let peer_pod = self
                    .devices
                    .borrow()
                    .get(&link_id.1)
                    .unwrap()
                    .meta()
                    .pod_name();

                let devices_ref = self.devices.borrow();
                let peer_links_ref = devices_ref.get(&link_id.1).unwrap().links();
                let peer_link = peer_links_ref.get(&(link_id.1, link_id.0)).unwrap().meta();

                let topo_link = link.meta().gen_topology_link(&peer_pod, peer_link);
                topology_links.push(topo_link);
            }

            topologies.push(Topology {
                metadata: Some(TopologyMeta {
                    name: dev.meta().pod_name(),
                }),
                spec: Some(TopologyLinks {
                    links: topology_links,
                }),
            })
        }

        EmunetReq { pods, topologies }
    }
}
