use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::server::{ContainerServer};

#[derive(Deserialize, Serialize)]
pub enum EmuNetState {
    Uninit,
    Working,
    Normal,
    Error,
}

#[derive(Deserialize, Serialize)]
pub struct EmuNet {
    name: String,
    capacity: u32,
    server_map: HashMap<Uuid, ContainerServer>,
    state: EmuNetState,
}

impl EmuNet {
    pub fn new(name: String, capacity: u32) -> Self {
        Self {
            name,
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
}

