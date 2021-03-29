use std::{cell::{Cell, RefCell}, collections::{hash_map::ValuesMut, HashMap}};

use serde::{Deserialize, Serialize};

use super::device::*;
use super::cluster::ContainerServer;

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

#[derive(Deserialize, Serialize)]
pub struct EmuNet {
    emunet_name: String,
    emunet_uuid: uuid::Uuid,
    max_capacity: u64,
    user_name: String,
    dev_count: Cell<u64>,
    servers: RefCell<HashMap<uuid::Uuid, ContainerServer>>,
    devices: RefCell<HashMap<u64, Device<String>>>,
}
