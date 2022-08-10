mod battery;
mod cpu;
mod error;
mod general;
mod gpu;

pub use battery::BatteryJson;
pub use cpu::CpuJson;
pub use general::{MinMaxJson, SettingsJson};
pub use gpu::GpuJson;

pub use error::JsonError;
