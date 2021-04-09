use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// LinkInfo represents an undirected edge connecting one node to another
// LinkInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct InputLink<T> {
    edge_id: (u64, u64),
    description: T,
}

impl<T> InputLink<T> {
    pub(crate) fn link_id(&self) -> (u64, u64) {
        self.edge_id
    }

    pub(crate) fn _meta(&self) -> &T {
        &self.description
    }
}

// DeviceInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct InputDevice<T> {
    id: u64,
    description: T,
}

impl<T> InputDevice<T> {
    pub(crate) fn id(&self) -> u64 {
        return self.id;
    }

    pub(crate) fn _meta(&self) -> &T {
        &self.description
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct InnerLink {
    pub(super) dest_dev_id: u64,
    pub(super) intf_name: String,
    pub(super) ip: String,
}

#[derive(Serialize)]
pub(crate) struct OutputDevice {
    pub(super) id: u64,
    pub(super) k8s_node_name: String,
    pub(super) k8s_pod_name: String,
    pub(super) pod_login_ip: Option<String>,
    pub(super) pod_login_user: Option<String>,
    pub(super) pod_login_pwd: Option<String>,
    pub(super) links: Vec<InnerLink>,
}

#[derive(Serialize)]
pub(crate) struct OutputLink {
    pub(super) link_id: (u64, u64),
    pub(super) details: HashMap<u64, InnerLink>,
}
