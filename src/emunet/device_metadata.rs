use std::cell::{Cell, RefCell};
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::k8s_api::{Pod, PodMeta, PodSpec, TopologyLink};

#[derive(Serialize, Deserialize)]
pub(crate) struct DeviceMeta {
    pub(crate) pod_name: String,
    pub(crate) k8s_node: String,
    pub(crate) intf_idx: Cell<u64>,
    pub(crate) login_ip: RefCell<Option<String>>,
    pub(crate) username: RefCell<Option<String>>,
    pub(crate) password: RefCell<Option<String>>,
}

impl DeviceMeta {
    pub(crate) fn new(k8s_node: &str, user_name: &str, emunet_name: &str, dev_id: u64) -> Self {
        Self {
            pod_name: format!("{}-{}-dev-{}", user_name, emunet_name, dev_id),
            k8s_node: k8s_node.to_string(),
            intf_idx: Cell::new(0),
            login_ip: RefCell::new(None),
            username: RefCell::new(None),
            password: RefCell::new(None),
        }
    }

    pub(crate) fn get_pod(&self) -> Pod {
        let metadata = PodMeta {
            name: self.pod_name.clone(),
        };
        let spec = PodSpec {
            node_selector: self.k8s_node.clone(),
        };

        Pod {
            metadata: Some(metadata),
            spec: Some(spec),
        }
    }

    pub(crate) fn get_intf_name(&self) -> String {
        let res = format!("intf-{}", self.intf_idx.get());
        self.intf_idx.set(self.intf_idx.get() + 1);
        res
    }

    pub(crate) fn pod_name(&self) -> &str {
        &self.pod_name
    }
}

impl DeviceMeta {
    pub(crate) fn udpate_login_info(&self, login_ip: &str, username: &str, password: &str) {
        *self.login_ip.borrow_mut() = Some(login_ip.to_string());
        *self.username.borrow_mut() = Some(username.to_string());
        *self.password.borrow_mut() = Some(password.to_string());
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LinkMeta {
    pub(crate) link_id: (u64, u64),
    pub(crate) link_uid: u32,
    pub(crate) intf: String,
    pub(crate) ip: String,
}

impl LinkMeta {
    pub(crate) fn new(
        link_id: (u64, u64),
        link_uid: u32,
        intf: String,
        ip: Ipv4Addr,
        subnet_len: u32,
    ) -> Self {
        LinkMeta {
            link_id,
            link_uid,
            intf,
            ip: format!("{}/{}", ip, subnet_len),
        }
    }

    pub(crate) fn gen_topology_link(
        &self,
        emunet_id: u8,
        peer_pod_name: &str,
        peer_link: &LinkMeta,
    ) -> TopologyLink {
        assert!(self.link_id.0 < (2 as u64).pow(32));
        assert!(self.link_id.1 < (2 as u64).pow(32));
        assert!(self.link_id.0 == peer_link.link_id.1);
        assert!(self.link_id.1 == peer_link.link_id.0);

        TopologyLink {
            uid: (((emunet_id as u32) << super::EDGES_POWER) | self.link_uid) as u64,
            peer_pod: peer_pod_name.to_string(),
            local_intf: self.intf.clone(),
            peer_intf: peer_link.intf.clone(),
            local_ip: self.ip.clone(),
            peer_ip: peer_link.ip.clone(),
        }
    }
}
