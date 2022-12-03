use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "target")]
pub enum CpuLimit {
    SteamDeck,
    SteamDeckAdvance,
    Generic(GenericCpuLimit),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericCpuLimit {
    /* TODO */
}
