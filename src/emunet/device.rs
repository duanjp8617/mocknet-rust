use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

// LinkInfo represents an undirected edge connecting one node to another
// LinkInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
pub(crate) struct LinkInfo<T> {
    edge_id: (u64, u64),
    description: T,
}

impl<T> LinkInfo<T> {
    pub(crate) fn link_id(&self) -> (u64, u64) {
        self.edge_id
    }

    pub(crate) fn _meta(&self) -> &T {
        &self.description
    }
}

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

// DeviceInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
pub(crate) struct DeviceInfo<T> {
    id: u64,
    description: T,
}

impl<T> DeviceInfo<T> {
    pub(crate) fn id(&self) -> u64 {
        return self.id;
    }

    pub(crate) fn _meta(&self) -> &T {
        &self.description
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
}
