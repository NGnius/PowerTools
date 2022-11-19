use std::convert::Into;

use crate::api::RangeLimit;
use crate::settings::{OnResume, OnSet, SettingError, SettingsRange};
use crate::settings::TBattery;
use crate::persist::BatteryJson;

#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate: Option<u64>,
    state: crate::state::steam_deck::Battery,
}

const BATTERY_VOLTAGE: f64 = 7.7;

const BATTERY_CHARGE_RATE_PATH: &str = "/sys/class/hwmon/hwmon5/maximum_battery_charge_rate"; // write-only
const BATTERY_CURRENT_NOW_PATH: &str = "/sys/class/power_supply/BAT1/current_now"; // read-only
const BATTERY_CHARGE_NOW_PATH: &str = "/sys/class/hwmon/hwmon2/device/charge_now"; // read-only
const BATTERY_CHARGE_FULL_PATH: &str = "/sys/class/hwmon/hwmon2/device/charge_full"; // read-only
const BATTERY_CHARGE_DESIGN_PATH: &str = "/sys/class/hwmon/hwmon2/device/charge_full_design"; // read-only

impl Battery {
    #[inline]
    pub fn from_json(other: BatteryJson, version: u64) -> Self {
        match version {
            0 => Self {
                charge_rate: other.charge_rate,
                state: crate::state::steam_deck::Battery::default(),
            },
            _ => Self {
                charge_rate: other.charge_rate,
                state: crate::state::steam_deck::Battery::default(),
            },
        }
    }

    fn set_all(&mut self) -> Result<(), SettingError> {
        if let Some(charge_rate) = self.charge_rate {
            self.state.charge_rate_set = true;
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, charge_rate).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: crate::settings::SettingVariant::Battery,
                },
            )
        } else if self.state.charge_rate_set {
            self.state.charge_rate_set = false;
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, Self::max().charge_rate.unwrap()).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: crate::settings::SettingVariant::Battery,
                },
            )
        } else {
            Ok(())
        }
    }

    fn clamp_all(&mut self) {
        let min = Self::min();
        let max = Self::max();
        if let Some(charge_rate) = &mut self.charge_rate {
            *charge_rate = (*charge_rate).clamp(min.charge_rate.unwrap(), max.charge_rate.unwrap());
        }
    }

    pub fn read_current_now() -> Result<u64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CURRENT_NOW_PATH) {
            Err((Some(e), None)) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CURRENT_NOW_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err((None, Some(e))) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CURRENT_NOW_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err(_) => panic!(
                "Invalid error while reading from `{}`",
                BATTERY_CURRENT_NOW_PATH
            ),
            // this value is in uA, while it's set in mA
            // so convert this to mA for consistency
            Ok(val) => Ok(val / 1000),
        }
    }

    pub fn read_charge_now() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CHARGE_NOW_PATH) {
            Err((Some(e), None)) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_NOW_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err((None, Some(e))) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_NOW_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err(_) => panic!(
                "Invalid error while reading from `{}`",
                BATTERY_CHARGE_NOW_PATH
            ),
            // convert to Wh
            Ok(val) => Ok((val as f64) / 1000000.0 * BATTERY_VOLTAGE),
        }
    }

    pub fn read_charge_full() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CHARGE_FULL_PATH) {
            Err((Some(e), None)) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_FULL_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err((None, Some(e))) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_FULL_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err(_) => panic!(
                "Invalid error while reading from `{}`",
                BATTERY_CHARGE_NOW_PATH
            ),
            // convert to Wh
            Ok(val) => Ok((val as f64) / 1000000.0 * BATTERY_VOLTAGE),
        }
    }

    pub fn read_charge_design() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CHARGE_DESIGN_PATH) {
            Err((Some(e), None)) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_DESIGN_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err((None, Some(e))) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_DESIGN_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            Err(_) => panic!(
                "Invalid error while reading from `{}`",
                BATTERY_CHARGE_NOW_PATH
            ),
            // convert to Wh
            Ok(val) => Ok((val as f64) / 1000000.0 * BATTERY_VOLTAGE),
        }
    }

    pub fn system_default() -> Self {
        Self {
            charge_rate: None,
            state: crate::state::steam_deck::Battery::default(),
        }
    }
}

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: self.charge_rate,
        }
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), SettingError> {
        self.clone().set_all()
    }
}

impl SettingsRange for Battery {
    #[inline]
    fn max() -> Self {
        Self {
            charge_rate: Some(2500),
            state: crate::state::steam_deck::Battery::default(),
        }
    }

    #[inline]
    fn min() -> Self {
        Self {
            charge_rate: Some(250),
            state: crate::state::steam_deck::Battery::default(),
        }
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        let max = Self::max();
        let min = Self::min();
        crate::api::BatteryLimits {
            charge_rate: Some(RangeLimit{
                min: min.charge_rate.unwrap(),
                max: max.charge_rate.unwrap(),
            }),
            charge_step: 50,
        }
    }

    fn json(&self) -> crate::persist::BatteryJson {
        self.clone().into()
    }

    fn charge_rate(&mut self, rate: Option<u64>) {
        self.charge_rate = rate;
    }

    fn get_charge_rate(&self) -> Option<u64> {
        self.charge_rate
    }
}
