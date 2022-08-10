use std::convert::Into;
use usdpl_back::core::serdes::Primitive;

use crate::settings::SettingError;

pub fn map_result<T: Into<Primitive>>(result: Result<T, SettingError>) -> super::ApiParameterType {
    match result {
        Ok(val) => vec![val.into()],
        Err(e) => vec![e.msg.into()],
    }
}

pub fn map_empty_result<T: Into<Primitive>>(
    result: Result<(), SettingError>,
    success: T,
) -> super::ApiParameterType {
    match result {
        Ok(_) => vec![success.into()],
        Err(e) => vec![e.msg.into()],
    }
}
