mod battery;
mod cpu;
mod error;
mod general;
mod gpu;
mod memory;
mod min_max;
mod traits;

pub use battery::Battery;
pub use cpu::Cpu;
pub use general::{SettingVariant, Settings, General};
pub use gpu::Gpu;
pub use memory::Memory;
pub use min_max::MinMax;
pub use memory::{TransparentHugepages, TransparentHugepagesParseError};

pub use error::SettingError;
pub use traits::{OnResume, OnSet, SettingsRange};

#[cfg(test)]
mod tests {
    #[test]
    fn system_defaults_test() {
        let settings = super::Settings::system_default("idc".into());
        println!("Loaded system settings: {:?}", settings);
    }
}
