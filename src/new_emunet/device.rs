use std::cell::RefCell;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct LinkInfo<T> {
    link_id: (u64, u64), // client side edge id in the form of (u64, u64)
    meta: T,             // a description string to hold the place
}

impl<T> LinkInfo<T> {
    pub fn new(link_id: (u64, u64), meta: T) -> Self {
        Self { link_id, meta }
    }

    pub fn link_id(&self) -> (u64, u64) {
        self.link_id
    }
}

// This represents a directed Link!!
#[derive(Deserialize, Serialize)]
pub struct Link<T> {
    link_uuid: (uuid::Uuid, uuid::Uuid), // out-going device -> incoming device
    meta: T,
}

impl<T> Link<T> {
    pub fn new(link_uuid: (uuid::Uuid, uuid::Uuid), meta: T) -> Self {
        Self { link_uuid, meta }
    }
}

impl<T> Link<T> {
    pub fn link_uuid(&self) -> &(uuid::Uuid, uuid::Uuid) {
        &self.link_uuid
    }
}

#[derive(Deserialize, Serialize)]
pub struct DeviceInfo<T> {
    id: u64,
    meta: T,
}

impl<T> DeviceInfo<T> {
    pub fn new(id: u64, meta: T) -> Self {
        Self { id, meta }
    }

    pub fn id(&self) -> u64 {
        return self.id;
    }
}

#[derive(Deserialize, Serialize)]
pub struct Device<T> {
    info: DeviceInfo<T>,
    server_uuid: uuid::Uuid,
    links: RefCell<HashMap<uuid::Uuid, Link<T>>>,
    // uuid is only reserved for compatibility reason
    uuid: uuid::Uuid,
}

impl<T> Device<T> {
    pub fn new(info: DeviceInfo<T>, server_uuid: uuid::Uuid) -> Self {
        Self {
            info,
            server_uuid,
            links: RefCell::new(HashMap::new()),
            uuid: indradb::util::generate_uuid_v1()
        }
    }

    pub fn add_link(&self, link: Link<T>) -> Result<(), String> {
        if self.uuid() != link.link_uuid.0 || self.links.borrow().contains_key(&link.link_uuid.1) {
            return Err("invalid link id".to_string());
        }
        if self.links.borrow_mut().insert(link.link_uuid.1.clone(), link).is_some() {
            panic!("fatal!".to_string());
        }
        Ok(())
    }
}

impl<T> Device<T> {
    pub fn id(&self) -> u64 {
        return self.info.id;
    }

    pub fn uuid(&self) -> uuid::Uuid {
        return self.uuid.clone();
    }

    pub fn device_info(&self) -> &DeviceInfo<T> {
        &self.info
    }

    pub fn server_uuid(&self) -> Uuid {
        self.server_uuid.clone()
    }

    pub fn links(&self) -> std::cell::Ref<HashMap<uuid::Uuid, Link<T>>>{
        self.links.borrow()
    }
}
