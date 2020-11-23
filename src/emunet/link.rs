use serde::{Deserialize, Serialize};
use super::server::{ServerPool, ContainerServer};

#[derive(Deserialize, Serialize)]
pub enum EmuNetState {
    Uninit,
    Working,
    Normal,
    Error,
}

#[derive(Deserialize, Serialize)]
pub struct EmuNetLink {
    name: String,
    capacity: u32,
    server_pool: ServerPool,
    state: EmuNetState,
}

impl EmuNet {
    pub fn new(name: String, capacity: u32) -> Self {
        Self {
            name,
            capacity,
            server_pool: ServerPool::new(),
            state: EmuNetState::Uninit,
        }
    }

    pub fn add_servers(&mut self, server_list: Vec<ContainerServer>) {
        self.server_pool.add_servers(server_list.into_iter());
    }

}

