use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmuNetUser {
    name: String,
}

impl EmuNetUser {
    pub fn new(name: String) -> Self {
        Self {name}
    }
}