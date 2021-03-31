use std::{
    cell::{Cell, RefCell},
    collections::{hash_map::ValuesMut, HashMap},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::cluster::ContainerServer;
use super::device::*;

pub static EMUNET_NODE_PROPERTY: &'static str = "default";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmunetError {
    PartitionFail(String),
    DatabaseFail(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmunetState {
    Uninit,
    Working,
    Normal,
    Error(EmunetError),
}

#[derive(Deserialize, Serialize)]
pub struct EmuNet {
    emunet_name: String,
    emunet_uuid: uuid::Uuid,
    max_capacity: u64,
    user_name: String,
    state: RefCell<EmunetState>,
    dev_count: Cell<u64>,
    servers: RefCell<HashMap<uuid::Uuid, ContainerServer>>,
    devices: RefCell<HashMap<u64, Device<String>>>,
}

impl EmuNet {
    pub fn new(
        emunet_name: String,
        emunet_uuid: Uuid,
        user_name: String,
        servers: Vec<ContainerServer>,
    ) -> Self {
        let (hm, max_capacity) =
            servers
                .into_iter()
                .fold((HashMap::new(), 0), |(mut hm, mut max_capacity), cs| {
                    max_capacity += cs.server_info().max_capacity;
                    let cs_uuid = cs.server_info().uuid.clone();
                    hm.insert(cs_uuid, cs);
                    (hm, max_capacity)
                });

        Self {
            emunet_name,
            emunet_uuid,
            max_capacity,
            user_name,
            state: RefCell::new(EmunetState::Uninit),
            dev_count: Cell::new(0),
            servers: RefCell::new(hm),
            devices: RefCell::new(HashMap::new()),
        }
    }
}

impl EmuNet {
    pub fn max_capacity(&self) -> u64 {
        self.max_capacity
    }
}

impl EmuNet {
    // modifying the state of the EmuNet
    pub fn state(&self) -> EmunetState {
        self.state.borrow().clone()
    }

    pub fn set_state(&self, state: EmunetState) {
        *self.state.borrow_mut() = state;
    }
}