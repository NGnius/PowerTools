use serde::{Deserialize, Serialize};

/// Message from the developers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeveloperMessage {
    /// Message identifier
    pub id: u64,
    /// Message title
    pub title: String,
    /// Message content
    pub body: String,
    /// Link for further information
    pub url: Option<String>,
}
