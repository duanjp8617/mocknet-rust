use std::cell::{Cell, Ref, RefMut};
use std::collections::HashMap;

use super::device::Device;
use super::emunet::Ipv4AddrAllocator;
use crate::k8s_api::{Pod, PodMeta, PodSpec};

pub(crate) struct DeviceMeta {
    pod: String,
    k8s_node: String,
    int_id_idx: Cell<u64>,
}

impl DeviceMeta {
    pub(crate) fn new(dev_id: u64, emunet_name: &str, k8s_node: String) -> Self {
        Self {
            pod: format!("{}-dev-{}", emunet_name, dev_id),
            k8s_node,
            int_id_idx: 1,
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

    fn gen_intf_name(&self) -> String {
        let res = format!("intf-{}", self.int_id_idx.get());
        self.int_id_idx.set(self.int_id_idx.get() + 1);
        res
    }
}

pub(crate) struct LinkMeta {
    uid: u64,
    intf: String,
    ip: String,
}

impl LinkMeta {
    pub(crate) fn new(
        link_id: (u64, u64),
        devices: Ref<HashMap<u64, Device<DeviceMeta, LinkMeta>>>,
        addr_allocator: Ref<Ipv4AddrAllocator>,
    ) -> Self {
        assert!(link_id.0 < (2 as u64).pow(32));
        assert!(link_id.1 < (2 as u64).pow(32));

        let uid = (link_id.0 << 32) & link_id.1;
        let intf = devices.get(&link_id.0).unwrap().meta().gen_intf_name();
        let ip = addr_allocator.try_alloc().unwrap();

        LinkMeta {
            uid,
            intf,
            ip: ip.to_string(),
        }
    }
}
