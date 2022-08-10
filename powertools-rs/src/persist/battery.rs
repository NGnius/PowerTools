use std::default::Default;
//use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BatteryJson {
    pub charge_rate: Option<u64>,
}

impl Default for BatteryJson {
    fn default() -> Self {
        Self { charge_rate: None }
    }
}
