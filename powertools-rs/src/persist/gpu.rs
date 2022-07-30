use std::default::Default;
//use std::fmt::Display;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GpuJson {
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
}

impl Default for GpuJson {
    fn default() -> Self {
        Self {
            fast_ppt: None,
            slow_ppt: None,
        }
    }
}
