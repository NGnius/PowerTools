use std::default::Default;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MemoryJson {
    pub transparent_hugepages: Option<String>,
}

impl Default for MemoryJson {
    fn default() -> Self {
        Self {
            transparent_hugepages: None,
        }
    }
}
