use serde::{Deserialize, Serialize};

use super::RangeLimit;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "target")]
pub enum CpuLimit {
    SteamDeck,
    SteamDeckAdvance,
    Generic(GenericCpuLimit),
    GenericAMD(GenericCpuLimit),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericCpuLimit {
    pub clock_min: Option<RangeLimit<u64>>,
    pub clock_max: Option<RangeLimit<u64>>,
    pub clock_step: u64,
}
