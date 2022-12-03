use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "target")]
pub enum GpuLimit {
    SteamDeck,
    SteamDeckAdvance,
    Generic(GenericGpuLimit),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericGpuLimit {
    /* TODO */
}
