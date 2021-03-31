use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
// LinkInfo represents an undirected edge connecting one node to another
// LinkInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
pub struct LinkInfo<T> {
    link_id: (u64, u64),
    meta: T,
}

impl<T> LinkInfo<T> {
    pub fn link_id(&self) -> (u64, u64) {
        self.link_id
    }

    pub fn meta(&self) -> &T {
        &self.meta
    }
}

// Link represents an directed edge from link_id.0 to link_id.1
#[derive(Deserialize, Serialize)]
pub struct Link<L> {
    link_id: (u64, u64),
    meta: L,
}

impl<L> Link<L> {
    pub fn new(source: u64, destination: u64, meta: L) -> Self {
        Self {
            link_id: (source, destination),
            meta,
        }
    }

    pub fn link_id(&self) -> (u64, u64) {
        self.link_id
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
pub struct DeviceInfo<T> {
    id: u64,
    meta: T,
}

impl<T> DeviceInfo<T> {
    pub fn id(&self) -> u64 {
        return self.id;
    }

    pub fn meta(&self) -> &T {
        &self.meta
    }
}

#[derive(Deserialize, Serialize)]
pub struct Device<D, L> {
    id: u64,
    server_uuid: uuid::Uuid,
    links: RefCell<HashSet<Link<L>>>,
    meta: D,
}

impl<D, L> Device<D, L> {
    pub fn new(id: u64, server_uuid: uuid::Uuid, meta: D) -> Self {
        Self {
            id,
            server_uuid,
            links: RefCell::new(HashSet::new()),
            meta,
        }
    }

    pub fn add_link(&self, link: Link<L>) -> bool {
        self.links.borrow_mut().insert(link)
    }
}

impl<D, L> Device<D, L> {
    pub fn id(&self) -> u64 {
        return self.id;
    }

    pub fn server_uuid(&self) -> Uuid {
        self.server_uuid.clone()
    }

    pub fn links(&self) -> std::cell::Ref<HashSet<Link<L>>> {
        self.links.borrow()
    }
}
