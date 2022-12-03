use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Target {
    SteamDeck,
    SteamDeckAdvance,
    Generic,
    Unknown,
}
