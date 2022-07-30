use std::convert::Into;

use crate::persist::CpuJson;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub online: bool,
    pub max_boost: u64,
    pub min_boost: u64,
    pub governor: String,
    pub boost: bool,
}

impl Cpu {
    #[inline]
    pub fn from_json(other: CpuJson, version: u64) -> Self {
        match version {
            0 => Self {
                online: other.online,
                max_boost: other.max_boost,
                min_boost: other.min_boost,
                governor: other.governor,
                boost: other.boost,
            },
            _ => Self {
                online: other.online,
                max_boost: other.max_boost,
                min_boost: other.min_boost,
                governor: other.governor,
                boost: other.boost,
            }
        }
    }
}

impl Into<CpuJson> for Cpu {
    #[inline]
    fn into(self) -> CpuJson {
        CpuJson {
            online: self.online,
            max_boost: self.max_boost,
            min_boost: self.min_boost,
            governor: self.governor,
            boost: self.boost,
        }
    }
}
