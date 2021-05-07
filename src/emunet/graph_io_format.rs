use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// LinkInfo represents an undirected edge connecting one node to another
// LinkInfo is deserialized from the incoming HTTP message
#[derive(Deserialize, Serialize)]
#[allow(dead_code)]
pub(crate) struct InputLink<T> {
    pub(crate) edge_id: (u64, u64),
    pub(crate) description: T,
}

impl<T> InputLink<T> {
    pub(crate) fn link_id(&self) -> (u64, u64) {
        self.edge_id
    }
}

// DeviceInfo is deserialized from the incoming HTTP message
#[derive(Deserialize, Serialize)]
#[allow(dead_code)]
pub(crate) struct InputDevice<T> {
    pub(crate) id: u64,
    pub(crate) description: T,
}

impl<T> InputDevice<T> {
    pub(crate) fn id(&self) -> u64 {
        return self.id;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct InnerLink {
    pub(crate) dest_dev_id: u64,
    pub(crate) intf_name: String,
    pub(crate) ip: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OutputDevice {
    pub(crate) id: u64,
    pub(crate) k8s_node_name: String,
    pub(crate) k8s_pod_name: String,
    pub(crate) pod_login_ip: Option<String>,
    pub(crate) pod_login_user: Option<String>,
    pub(crate) pod_login_pwd: Option<String>,
    pub(crate) links: Vec<InnerLink>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OutputLink {
    pub(crate) link_id: (u64, u64),
    pub(crate) details: HashMap<u64, InnerLink>,
}
