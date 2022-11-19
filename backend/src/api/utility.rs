use std::convert::Into;
use usdpl_back::core::serdes::Primitive;

use crate::settings::SettingError;

#[inline]
pub fn map_result<T: Into<Primitive>>(result: Result<T, SettingError>) -> super::ApiParameterType {
    match result {
        Ok(val) => vec![val.into()],
        Err(e) => {
            log::debug!("Mapping error to primitive: {}", e);
            vec![e.msg.into()]
        },
    }
}

#[inline]
pub fn map_optional_result<T: Into<Primitive>>(result: Result<Option<T>, SettingError>) -> super::ApiParameterType {
    match result {
        Ok(val) => match val {
            Some(val) => vec![val.into()],
            None => vec![Primitive::Empty],
        },
        Err(e) => {
            log::debug!("Mapping error to primitive: {}", e);
            vec![e.msg.into()]
        },
    }
}

/*#[inline]
pub fn map_empty_result<T: Into<Primitive>>(
    result: Result<(), SettingError>,
    success: T,
) -> super::ApiParameterType {
    match result {
        Ok(_) => vec![success.into()],
        Err(e) => {
            log::debug!("Mapping error to primitive: {}", e);
            vec![e.msg.into()]
        },
    }
}*/
