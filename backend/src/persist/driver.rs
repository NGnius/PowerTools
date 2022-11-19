//use std::default::Default;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum DriverJson {
    #[default]
    #[serde(rename = "steam-deck", alias = "gabe-boy")]
    SteamDeck,
    #[serde(rename = "steam-deck-oc", alias = "gabe-boy-advance")]
    SteamDeckAdvance,
    #[serde(rename = "unknown")]
    Unknown,
}
