use serde::{Deserialize, Serialize};

/// Base JSON limits information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RangeLimit<T> {
    pub min: T,
    pub max: T,
}
