//use std::default::Default;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum DriverJson {
    #[serde(rename = "steam-deck", alias = "gabe-boy")]
    SteamDeck,
    #[serde(rename = "steam-deck-oc", alias = "gabe-boy-advance")]
    SteamDeckAdvance,
    #[serde(rename = "generic")]
    Generic,
    #[serde(rename = "generic-amd")]
    GenericAMD,
    #[serde(rename = "unknown")]
    Unknown,
    #[default]
    #[serde(rename = "auto")]
    AutoDetect,
}
