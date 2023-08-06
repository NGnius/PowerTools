use std::default::Default;
//use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BatteryJson {
    pub charge_rate: Option<u64>,
    pub charge_mode: Option<String>,
    #[serde(default)]
    pub events: Vec<BatteryEventJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BatteryEventJson {
    pub trigger: String,
    pub charge_rate: Option<u64>,
    pub charge_mode: Option<String>,
}

impl Default for BatteryJson {
    fn default() -> Self {
        Self {
            charge_rate: None,
            charge_mode: None,
            events: Vec::new(),
            root: None,
        }
    }
}
