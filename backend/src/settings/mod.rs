mod detect;
pub mod driver;
mod error;
mod general;
mod min_max;
mod traits;
mod util;

pub mod generic;
pub mod generic_amd;
pub mod steam_deck;
pub mod unknown;

pub use detect::{auto_detect0, auto_detect_provider, limits_worker::spawn as limits_worker_spawn};
pub use driver::Driver;
pub use general::{SettingVariant, Settings, General};
pub use min_max::{MinMax, min_max_from_json};

pub use error::SettingError;
pub use traits::{OnResume, OnSet, TGeneral, TGpu, TCpus, TBattery, TCpu, OnPowerEvent, PowerMode};

#[cfg(test)]
mod tests {
    #[test]
    fn system_defaults_test() {
        let settings = super::Settings::system_default("idc".into(), "Cool name".into());
        println!("Loaded system settings: {:?}", settings);
    }
}
