pub mod battery;
pub mod cpu;
pub mod general;
pub mod gpu;
pub mod memory;
mod utility;

pub(super) type ApiParameterType = Vec<usdpl_back::core::serdes::Primitive>;
