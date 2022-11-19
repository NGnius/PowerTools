mod api_types;
pub mod battery;
pub mod cpu;
pub mod general;
pub mod gpu;
pub mod handler;
mod async_utils;
mod utility;

pub(super) type ApiParameterType = Vec<usdpl_back::core::serdes::Primitive>;

pub use api_types::*;
