use std::default::Default;
//use std::fmt::Display;

use serde::{Serialize, Deserialize};

const SCALING_FREQUENCIES: &[u64] = &[1700000, 2400000, 2800000];

#[derive(Serialize, Deserialize)]
pub struct CpuJson {
    pub online: bool,
    pub max_boost: u64,
    pub min_boost: u64,
    pub governor: String,
    pub boost: bool,
}

impl Default for CpuJson {
    fn default() -> Self {
        Self {
            online: true,
            max_boost: SCALING_FREQUENCIES[SCALING_FREQUENCIES.len() - 1],
            min_boost: SCALING_FREQUENCIES[0],
            governor: "schedutil".to_owned(),
            boost: true,
        }
    }
}
