pub mod battery;
pub mod cpu;
pub mod general;
pub mod gpu;
mod utility;

pub(super) type ApiParameterType = Vec<usdpl_back::core::serdes::Primitive>;
