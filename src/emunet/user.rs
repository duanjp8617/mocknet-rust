use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmuNetUser {
    name: String,
    fuck: i32,
}

impl EmuNetUser {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fuck: 5,
        }
    }
}