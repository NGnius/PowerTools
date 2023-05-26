use serde::{Deserialize, Serialize};

/// Base JSON limits information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RangeLimit<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}
