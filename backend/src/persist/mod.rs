mod battery;
mod cpu;
mod driver;
mod error;
mod general;
mod gpu;

pub use battery::{BatteryEventJson, BatteryJson};
pub use cpu::CpuJson;
pub use driver::DriverJson;
pub use general::{MinMaxJson, OnEventJson, SettingsJson};
pub use gpu::GpuJson;

pub use error::JsonError;
