use std::collections::{HashMap, hash_map::ValuesMut};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::server::{ContainerServer};

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
    state: EmuNetState,
    server_map: HashMap<Uuid, ContainerServer>,
    vertex_map: HashMap<u64, Uuid>,
}

impl EmuNet {
    pub fn new(name: String, uuid: Uuid, capacity: u32) -> Self {
        Self {
            name,
            uuid,
            capacity,
            state: EmuNetState::Uninit,
            server_map: HashMap::new(),
            vertex_map: HashMap::new(),
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

    pub fn add_vertex_assignment(&mut self, vertex_client_id: u64, vertex_uuid: Uuid) {
        self.vertex_map.insert(vertex_client_id, vertex_uuid);
    }
}

impl EmuNet {
    // modifying the state of the EmuNet
    pub fn state(&self) -> EmuNetState {
        self.state.clone()
    }
}
