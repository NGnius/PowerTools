use std::convert::Into;

use crate::api::RangeLimit;
use crate::settings::{OnResume, OnSet, SettingError, SettingsRange};
use crate::settings::TBattery;
use crate::persist::BatteryJson;
use super::util::ChargeMode;

#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate: Option<u64>,
    pub charge_mode: Option<ChargeMode>,
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
                charge_mode: other.charge_mode.map(|x| Self::str_to_charge_mode(&x)).flatten(),
                state: crate::state::steam_deck::Battery::default(),
            },
            _ => Self {
                charge_rate: other.charge_rate,
                charge_mode: other.charge_mode.map(|x| Self::str_to_charge_mode(&x)).flatten(),
                state: crate::state::steam_deck::Battery::default(),
            },
        }
    }

    #[inline]
    fn charge_mode_to_str(mode: ChargeMode) -> String {
        match mode {
            ChargeMode::Normal => "normal",
            ChargeMode::Idle => "idle",
            ChargeMode::Discharge => "discharge",
        }.to_owned()
    }

    #[inline]
    fn str_to_charge_mode(s: &str) -> Option<ChargeMode> {
        match s {
            "normal" => Some(ChargeMode::Normal),
            "idle" => Some(ChargeMode::Idle),
            "discharge" | "disacharge" => Some(ChargeMode::Discharge),
            _ => None,
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
            )?;
        } else if self.state.charge_rate_set {
            self.state.charge_rate_set = false;
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, Self::max().charge_rate.unwrap()).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: crate::settings::SettingVariant::Battery,
                },
            )?;
        }
        if let Some(charge_mode) = self.charge_mode {
            self.state.charge_mode_set = true;
            super::util::set(super::util::Setting::ChargeMode, charge_mode as _).map_err(
                |e| SettingError {
                    msg: format!("Failed to set charge mode: {}", e),
                    setting: crate::settings::SettingVariant::Battery,
                },
            )?;
        } else if self.state.charge_mode_set {
            self.state.charge_mode_set = false;
            super::util::set(super::util::Setting::ChargeMode, ChargeMode::Normal as _).map_err(
                |e| SettingError {
                    msg: format!("Failed to set charge mode: {}", e),
                    setting: crate::settings::SettingVariant::Battery,
                },
            )?;
        }
        Ok(())
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
            charge_mode: None,
            state: crate::state::steam_deck::Battery::default(),
        }
    }
}

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: self.charge_rate,
            charge_mode: self.charge_mode.map(Self::charge_mode_to_str),
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
            charge_mode: None,
            state: crate::state::steam_deck::Battery::default(),
        }
    }

    #[inline]
    fn min() -> Self {
        Self {
            charge_rate: Some(250),
            charge_mode: None,
            state: crate::state::steam_deck::Battery::default(),
        }
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        let max = Self::max();
        let min = Self::min();
        crate::api::BatteryLimits {
            charge_current: Some(RangeLimit{
                min: min.charge_rate.unwrap(),
                max: max.charge_rate.unwrap(),
            }),
            charge_current_step: 50,
            charge_modes: vec!["discharge".to_owned(), "idle".to_owned(), "normal".to_owned()],
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

    fn charge_mode(&mut self, mode: Option<String>) {
        self.charge_mode = mode.map(|s| Self::str_to_charge_mode(&s)).flatten()
    }

    fn get_charge_mode(&self) -> Option<String> {
        self.charge_mode.map(Self::charge_mode_to_str)
    }
}
