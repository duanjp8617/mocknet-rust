use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use super::device::Device;
use crate::k8s_api::{Pod, PodMeta, PodSpec, TopologyLink};

#[derive(Serialize, Deserialize)]
pub(crate) struct DeviceMeta {
    pod: String,
    k8s_node: String,
    int_id_idx: Cell<u64>,
    login_ip: RefCell<Option<String>>,
    username: RefCell<Option<String>>,
    password: RefCell<Option<String>>,
}

impl DeviceMeta {
    pub(crate) fn new(dev_id: u64, emunet_name: &str, k8s_node: &str) -> Self {
        Self {
            pod: format!("{}-dev-{}", emunet_name, dev_id),
            k8s_node: k8s_node.to_string(),
            int_id_idx: Cell::new(0),
            login_ip: RefCell::new(None),
            username: RefCell::new(None),
            password: RefCell::new(None),
        }
    }

    pub(crate) fn get_pod(&self) -> Pod {
        let metadata = PodMeta {
            name: self.pod.clone(),
        };
        let spec = PodSpec {
            node_selector: self.k8s_node.clone(),
        };

        Pod {
            metadata: Some(metadata),
            spec: Some(spec),
        }
    }

    pub(crate) fn generate_intf_name(&self) -> String {
        let res = format!("intf-{}", self.int_id_idx.get());
        self.int_id_idx.set(self.int_id_idx.get() + 1);
        res
    }

    pub(crate) fn pod_name(&self) -> String {
        self.pod.clone()
    }
}

impl DeviceMeta {
    pub(crate) fn udpate_info(&self, login_ip: &str, username: &str, password: &str) {
        *self.login_ip.borrow_mut() = Some(login_ip.to_string());
        *self.username.borrow_mut() = Some(username.to_string());
        *self.password.borrow_mut() = Some(password.to_string());
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LinkMeta {
    link_id: (u64, u64),
    intf: String,
    ip: String,
}

impl LinkMeta {
    pub(crate) fn new(
        source: u64,
        destination: u64,
        intf: String,
        ip: Ipv4Addr,
        subnet_len: u32,
    ) -> Self {
        LinkMeta {
            link_id: (source, destination),
            intf,
            ip: format!("{}/{}", ip, subnet_len),
        }
    }

    pub(crate) fn gen_topology_link(&self, peer_pod: &str, peer_link: &LinkMeta) -> TopologyLink {
        assert!(self.link_id.0 < (2 as u64).pow(32));
        assert!(self.link_id.1 < (2 as u64).pow(32));
        assert!(self.link_id.0 == peer_link.link_id.1);
        assert!(self.link_id.1 == peer_link.link_id.0);

        TopologyLink {
            uid: (self.link_id.0 << 32) | self.link_id.1,
            peer_pod: peer_pod.to_string(),
            local_intf: self.intf.clone(),
            peer_intf: peer_link.intf.clone(),
            local_ip: self.ip.clone(),
            peer_ip: peer_link.ip.clone(),
        }
    }
}
