mod battery;
mod cpu;
mod error;
mod general;
mod gpu;
mod memory;

pub use battery::BatteryJson;
pub use cpu::CpuJson;
pub use general::{MinMaxJson, SettingsJson};
pub use gpu::GpuJson;
pub use memory::MemoryJson;

pub use error::JsonError;
