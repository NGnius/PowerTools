use serde::{Deserialize, Serialize};
use super::RangeLimit;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "target")]
pub enum GpuLimit {
    SteamDeck,
    SteamDeckAdvance,
    Generic(GenericGpuLimit),
    GenericAMD(GenericGpuLimit),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericGpuLimit {
    pub fast_ppt: Option<RangeLimit<u64>>,
    pub slow_ppt: Option<RangeLimit<u64>>,
    pub ppt_step: Option<u64>,
    pub tdp: Option<RangeLimit<u64>>,
    pub tdp_boost: Option<RangeLimit<u64>>,
    pub tdp_step: Option<u64>,
    pub clock_min: Option<RangeLimit<u64>>,
    pub clock_max: Option<RangeLimit<u64>>,
    pub clock_step: Option<u64>,
}
