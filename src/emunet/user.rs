use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use super::emunet::Emunet;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Retired {
    pub(crate) version: u64,
    pub(crate) name: String,
    pub(crate) nodes: Vec<u64>,
    pub(crate) edges: Vec<(u64, u64)>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct User {
    name: String,
    emunet_name_to_uuid: RefCell<HashMap<String, uuid::Uuid>>,
    retired: RefCell<Vec<Retired>>,
}

impl User {
    pub(crate) fn new<S: std::convert::Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            emunet_name_to_uuid: RefCell::new(HashMap::new()),
            retired: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn register_emunet<S: std::convert::Into<String>>(
        &self,
        emunet_name: S,
    ) -> Option<uuid::Uuid> {
        let emunet_name = emunet_name.into();
        if self
            .emunet_name_to_uuid
            .borrow()
            .get(&emunet_name)
            .is_some()
        {
            None
        } else {
            let uuid = indradb::util::generate_uuid_v1();
            self.emunet_name_to_uuid
                .borrow_mut()
                .insert(emunet_name, uuid.clone());
            Some(uuid)
        }
    }

    pub(crate) fn delete_emunet(&self, emunet_name: &str) -> Option<uuid::Uuid> {
        self.emunet_name_to_uuid.borrow_mut().remove(emunet_name)
    }

    pub(crate) fn add_retired(&self, emunet: &Emunet) {
        let history = emunet.release_history();
        self.retired.borrow_mut().push(Retired {
            version: history.0,
            name: history.1,
            nodes: history.2,
            edges: history.3,
        });
    }
}

impl User {
    pub(crate) fn into_uuid_map(self) -> HashMap<String, uuid::Uuid> {
        let hm = std::mem::replace(
            &mut (*self.emunet_name_to_uuid.borrow_mut()),
            HashMap::new(),
        );
        hm
    }

    pub(crate) fn get_retired_emunets(&self) -> Vec<Retired> {
        self.retired.borrow().clone()
    }
}
