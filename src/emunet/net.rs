use std::collections::{HashMap, hash_map::ValuesMut};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::server::{ContainerServer};
use crate::algo::in_memory_graph::InMemoryGraph;
use crate::algo::Partition;

#[derive(Deserialize, Serialize, Clone, Debug)]
struct VDevice {
    id: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Vlink {
    edge_id: (u64, u64)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmuNetState {
    Uninit,
    Working,
    Normal,
    Error,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EmuNet {
    name: String,
    uuid: Uuid,
    capacity: u32,
    server_map: HashMap<Uuid, ContainerServer>,
    state: EmuNetState,
}

impl EmuNet {
    pub fn new(name: String, uuid: Uuid, capacity: u32) -> Self {
        Self {
            name,
            uuid,
            capacity,
            server_map: HashMap::new(),
            state: EmuNetState::Uninit,
        }
    }

    pub fn add_servers(&mut self, server_list: Vec<ContainerServer>) {
        for cs in server_list.into_iter() {
            let server_id = cs.id();
            if self.server_map.contains_key(&server_id) {
                panic!("collisioned server key, this should never happen");
            }
            self.server_map.insert(server_id, cs);
        }
    }

    pub fn servers_mut<'a>(&'a mut self) ->  ValuesMut<'a, Uuid, ContainerServer>{
        self.server_map.values_mut()
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }
}

