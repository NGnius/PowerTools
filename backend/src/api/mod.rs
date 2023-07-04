mod api_types;
mod async_utils;
pub mod battery;
pub mod cpu;
pub mod general;
pub mod gpu;
pub mod handler;
pub mod message;
mod utility;

pub(super) type ApiParameterType = Vec<usdpl_back::core::serdes::Primitive>;

pub use api_types::*;
