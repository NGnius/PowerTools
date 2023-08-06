use std::default::Default;
//use std::fmt::Display;

use super::MinMaxJson;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GpuJson {
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
    pub clock_limits: Option<MinMaxJson<u64>>,
    pub slow_memory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}

impl Default for GpuJson {
    fn default() -> Self {
        Self {
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            slow_memory: false,
            root: None,
        }
    }
}
