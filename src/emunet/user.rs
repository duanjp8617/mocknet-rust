use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub(crate) struct User {
    name: String,
    emunet_name_to_uuid: RefCell<HashMap<String, uuid::Uuid>>,
}

impl User {
    pub(crate) fn new<S: std::convert::Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            emunet_name_to_uuid: RefCell::new(HashMap::new()),
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
}

impl User {
    pub(crate) fn into_uuid_map(self) -> HashMap<String, uuid::Uuid> {
        let hm = std::mem::replace(
            &mut (*self.emunet_name_to_uuid.borrow_mut()),
            HashMap::new(),
        );
        hm
    }
}
