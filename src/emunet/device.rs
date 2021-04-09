use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use super::device_metadata::{DeviceMeta, LinkMeta};
use super::graph_io_format::{InnerLink, OutputDevice};

// Link represents an directed edge from link_id.0 to link_id.1
#[derive(Deserialize, Serialize)]
pub(crate) struct Link<L> {
    link_id: (u64, u64),
    meta: L,
}

impl<L> Link<L> {
    pub(crate) fn new(source: u64, destination: u64, meta: L) -> Self {
        Self {
            link_id: (source, destination),
            meta,
        }
    }

    pub(crate) fn link_id(&self) -> (u64, u64) {
        self.link_id
    }

    pub(crate) fn meta(&self) -> &L {
        &self.meta
    }
}

// necessary trait implemenation to make Link HashSet compatible
impl<L> Borrow<(u64, u64)> for Link<L> {
    fn borrow(&self) -> &(u64, u64) {
        &self.link_id
    }
}

impl<L> PartialEq for Link<L> {
    fn eq(&self, other: &Link<L>) -> bool {
        self.link_id.eq(&other.link_id)
    }
}

impl<L> Eq for Link<L> {}

impl<L> Hash for Link<L> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.link_id.hash(state);
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Device<D, L> {
    id: u64,
    server_name: String,
    links: RefCell<HashSet<Link<L>>>,
    meta: D,
}

impl<D, L> Device<D, L> {
    pub(crate) fn new(id: u64, server_name: String, meta: D) -> Self {
        Self {
            id,
            server_name,
            links: RefCell::new(HashSet::new()),
            meta,
        }
    }

    pub(crate) fn add_link(&self, link: Link<L>) -> bool {
        self.links.borrow_mut().insert(link)
    }
}

#[allow(dead_code)]
impl<D, L> Device<D, L> {
    pub(crate) fn server_name(&self) -> String {
        self.server_name.clone()
    }

    pub(crate) fn links(&self) -> std::cell::Ref<HashSet<Link<L>>> {
        self.links.borrow()
    }

    pub(crate) fn meta(&self) -> &D {
        return &self.meta;
    }

    pub(crate) fn id(&self) -> u64 {
        return self.id;
    }
}

impl Device<DeviceMeta, LinkMeta> {
    pub(crate) fn get_output_device(&self) -> OutputDevice {
        let mut links = Vec::new();
        for link in self.links().iter() {
            let inner = InnerLink {
                dest_dev_id: link.link_id.1,
                intf_name: link.meta().intf.clone(),
                ip: link.meta().ip.clone(),
            };
            links.push(inner);
        }
        links.sort_by(|l1, l2| l1.dest_dev_id.cmp(&l2.dest_dev_id));

        OutputDevice {
            id: self.id,
            k8s_node_name: self.server_name.clone(),
            k8s_pod_name: self.meta().pod_name.clone(),
            pod_login_ip: self.meta().login_ip.borrow().as_ref().map(|s| s.clone()),
            pod_login_user: self.meta().username.borrow().as_ref().map(|s| s.clone()),
            pod_login_pwd: self.meta().password.borrow().as_ref().map(|s| s.clone()),
            links,
        }
    }

    pub(crate) fn get_inner_link(&self, dest_id: u64) -> Option<InnerLink> {
        self.links
            .borrow()
            .get(&(self.id, dest_id))
            .map(|link| InnerLink {
                dest_dev_id: link.link_id.1,
                intf_name: link.meta().intf.clone(),
                ip: link.meta().ip.clone(),
            })
    }
}
