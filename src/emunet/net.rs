use serde::{Deserialize, Serialize};
use super::server::{ServerPool, ContainerServer};

#[derive(Deserialize, Serialize)]
pub struct EmuNet {
    name: String,
    capacity: u32,
    server_pool: ServerPool,
}

impl EmuNet {
    pub fn new(name: String, capacity: u32) -> Self {
        Self {
            name,
            capacity,
            server_pool: ServerPool::new(),
        }
    }

    pub fn add_servers(&mut self, server_list: Vec<ContainerServer>) {
        self.server_pool.add_servers(server_list.into_iter());
    }

}

