mod battery;
mod cpu;
mod error;
mod general;
mod gpu;
mod min_max;
mod traits;

pub use battery::Battery;
pub use cpu::{Cpu, Cpus};
pub use general::{SettingVariant, Settings, General};
pub use gpu::Gpu;
pub use min_max::MinMax;

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
