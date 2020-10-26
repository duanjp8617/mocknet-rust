use serde::{Deserialize, Serialize};
use super::server::ServerPool;

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
            server_pool: ServerPool::new()
        }
    }
}

