use std::default::Default;
//use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::MinMaxJson;

//const SCALING_FREQUENCIES: &[u64] = &[1700000, 2400000, 2800000];

#[derive(Serialize, Deserialize)]
pub struct CpuJson {
    pub online: bool,
    pub clock_limits: Option<MinMaxJson<u64>>,
    pub governor: String,
}

impl Default for CpuJson {
    fn default() -> Self {
        Self {
            online: true,
            clock_limits: None,
            governor: "schedutil".to_owned(),
        }
    }
}
