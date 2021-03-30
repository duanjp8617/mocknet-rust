use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, cell::RefCell};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct User {
    name: String,
    emunet_name_to_uuid: RefCell<HashMap<String, uuid::Uuid>>,
}

impl User {
    pub fn new<S: std::convert::Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            emunet_name_to_uuid: RefCell::new(HashMap::new()),
        }
    }

    pub fn register_emunet<S: std::convert::Into<String>>(
        &self,
        emunet_name: S,
    ) -> Option<uuid::Uuid> {
        let emunet_name = emunet_name.into();
        if self.emunet_name_to_uuid.borrow().get(&emunet_name).is_some() {
            None
        } else {
            let uuid = indradb::util::generate_uuid_v1();
            self.emunet_name_to_uuid.borrow_mut().insert(emunet_name, uuid.clone());
            Some(uuid)
        }
    }

    pub fn delete_emunet(&self, emunet_name: &str) -> Option<uuid::Uuid> {
        self.emunet_name_to_uuid.borrow_mut().remove(emunet_name)
    }
}

impl User {
    pub fn get_emunet_uuid_map(&self) -> HashMap<String, uuid::Uuid> {
        return self.emunet_name_to_uuid.borrow().clone();
    }

    pub fn into_uuid_map(self) -> HashMap<String, uuid::Uuid> {
        let hm = std::mem::replace(&mut (*self.emunet_name_to_uuid.borrow_mut()), HashMap::new());
        hm
    }
}
