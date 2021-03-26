use std::collections::{hash_map::ValuesMut, HashMap};

use serde::{Deserialize, Serialize};

use super::device::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmuNetError {
    PartitionFail(String),
    DatabaseFail(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmuNetState {
    Uninit,
    Working,
    Normal,
    Error(EmuNetError),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EmuNet {
    emunet_name: String,
    emunet_uuid: uuid::Uuid,

    user_name: String,
    name: String,
    uuid: Uuid,
    capacity: u32,
    max_capacity: u32,
    state: EmuNetState,
    server_map: HashMap<Uuid, ContainerServer>,
    vertex_map: HashMap<u64, Uuid>,
    vertex_assignment: HashMap<u64, Uuid>,
}
