use std::convert::Into;

use crate::persist::MinMaxJson;

#[derive(Debug, Clone)]
pub struct MinMax<T> {
    pub max: T,
    pub min: T,
}

impl<T> MinMax<T> {
    #[inline]
    pub fn from_json<X: Into<T>>(other: MinMaxJson<X>, _version: u64) -> Self {
        Self {
            max: other.max.into(),
            min: other.min.into(),
        }
    }
}

impl<X: Into<Y>, Y> Into<MinMaxJson<Y>> for MinMax<X> {
    #[inline]
    fn into(self) -> MinMaxJson<Y> {
        MinMaxJson {
            max: self.max.into(),
            min: self.min.into(),
        }
    }
}
