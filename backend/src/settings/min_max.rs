use std::convert::Into;

use limits_core::json::RangeLimit;

use crate::persist::MinMaxJson;

pub type MinMax<T> = RangeLimit<T>;

pub fn min_max_from_json<T, X: Into<T>>(other: MinMaxJson<X>, _version: u64) -> MinMax<T> {
    MinMax {
        max: other.max.into(),
        min: other.min.into(),
    }
}

impl<X: Into<Y>, Y> Into<MinMaxJson<Y>> for RangeLimit<X> {
    #[inline]
    fn into(self) -> MinMaxJson<Y> {
        MinMaxJson {
            max: self.max.into(),
            min: self.min.into(),
        }
    }
}
