mod battery;
mod cpu;
mod driver;
mod error;
mod general;
mod gpu;

pub use battery::BatteryJson;
pub use cpu::CpuJson;
pub use driver::DriverJson;
pub use general::{MinMaxJson, SettingsJson};
pub use gpu::GpuJson;

pub use error::JsonError;
