use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "target")]
pub enum BatteryLimit {
    SteamDeck,
    SteamDeckAdvance,
    Generic(GenericBatteryLimit),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericBatteryLimit {
    /* TODO */
}
