use crate::persist::GpuJson;
use crate::settings::MinMax;
use crate::settings::generic::Gpu as GenericGpu;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::TGpu;

#[derive(Debug)]
pub struct Gpu {
    generic: GenericGpu,
}

impl Gpu {
    pub fn from_limits(limits: limits_core::json::GenericGpuLimit) -> Self {
        Self {
            generic: GenericGpu::from_limits(limits),
        }
    }

    pub fn from_json_and_limits(other: GpuJson, version: u64, limits: limits_core::json::GenericGpuLimit) -> Self {
        Self {
            generic: GenericGpu::from_json_and_limits(other, version, limits),
        }
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), SettingError> {
        self.generic.on_resume()
        // TODO
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.generic.on_set()
        // TODO
    }
}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        self.generic.limits()
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.generic.json()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        self.generic.ppt(fast, slow)
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        self.generic.get_ppt()
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.generic.clock_limits(limits)
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.generic.get_clock_limits()
    }

    fn slow_memory(&mut self) -> &mut bool {
        self.generic.slow_memory()
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::GenericAMD
    }
}
