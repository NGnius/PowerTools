use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    pub conditions: super::Conditions,
    pub limits: Vec<super::Limits>,
}
