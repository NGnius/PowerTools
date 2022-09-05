use std::convert::Into;

use super::{OnResume, OnSet, SettingError, SettingsRange};
use crate::persist::BatteryJson;

#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate: Option<u64>,
    state: crate::state::Battery,
}

const BATTERY_CHARGE_RATE_PATH: &str = "/sys/class/hwmon/hwmon5/maximum_battery_charge_rate"; // write-only
const BATTERY_CURRENT_NOW_PATH: &str = "/sys/class/power_supply/BAT1/current_now"; // read-only

impl Battery {
    #[inline]
    pub fn from_json(other: BatteryJson, version: u64) -> Self {
        match version {
            0 => Self {
                charge_rate: other.charge_rate,
                state: crate::state::Battery::default(),
            },
            _ => Self {
                charge_rate: other.charge_rate,
                state: crate::state::Battery::default(),
            },
        }
    }

    fn set_all(&mut self) -> Result<(), SettingError> {
        if let Some(charge_rate) = self.charge_rate {
            self.state.charge_rate_set = true;
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, charge_rate).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: super::SettingVariant::Battery,
                },
            )
        } else if self.state.charge_rate_set {
            self.state.charge_rate_set = false;
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, Self::max().charge_rate.unwrap()).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: super::SettingVariant::Battery,
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

    pub fn current_now() -> Result<u64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CURRENT_NOW_PATH) {
            Err((Some(e), None)) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CURRENT_NOW_PATH, e),
                setting: super::SettingVariant::Battery,
            }),
            Err((None, Some(e))) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CURRENT_NOW_PATH, e),
                setting: super::SettingVariant::Battery,
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

    pub fn system_default() -> Self {
        Self {
            charge_rate: None,
            state: crate::state::Battery::default(),
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
            state: crate::state::Battery::default(),
        }
    }

    #[inline]
    fn min() -> Self {
        Self {
            charge_rate: Some(250),
            state: crate::state::Battery::default(),
        }
    }
}
