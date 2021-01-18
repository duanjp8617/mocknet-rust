use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct EmuNetUser {
    name: String,
    emu_net_ids: HashMap<String, Uuid>,
}

impl EmuNetUser {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            emu_net_ids: HashMap::new(),
        }
    }

    pub fn add_emu_net(&mut self, emu_net_name: String, emu_net_id: Uuid) -> bool {
        if self.emu_net_ids.get(&emu_net_name).is_some() {
            false
        } else {
            self.emu_net_ids.insert(emu_net_name, emu_net_id);
            true
        }
    }

    pub fn emu_net_exist(&self, emu_net_name: &str) -> bool {
        self.emu_net_ids.get(emu_net_name).is_some()
    }

    pub fn get_all_emu_nets(&self) -> HashMap<String, Uuid> {
        return self.emu_net_ids.clone();
    }
}
