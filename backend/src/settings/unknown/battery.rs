use std::convert::Into;

use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::TBattery;
use crate::persist::BatteryJson;

#[derive(Debug, Clone)]
pub struct Battery;

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: None,
        }
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), SettingError> {
        Ok(())
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), SettingError> {
        Ok(())
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        crate::api::BatteryLimits {
            charge_rate: None,
            charge_step: 50,
        }
    }

    fn json(&self) -> crate::persist::BatteryJson {
        self.clone().into()
    }

    fn charge_rate(&mut self, _rate: Option<u64>) {
    }

    fn get_charge_rate(&self) -> Option<u64> {
        None
    }
}
