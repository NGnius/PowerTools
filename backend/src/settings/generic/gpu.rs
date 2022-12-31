use std::convert::Into;

use limits_core::json::GenericGpuLimit;

use crate::settings::MinMax;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::TGpu;
use crate::persist::GpuJson;

#[derive(Debug, Clone)]
pub struct Gpu {
    slow_memory: bool, // ignored
    limits: GenericGpuLimit,
}

impl Gpu {
    /*#[inline]
    pub fn from_json(_other: GpuJson, _version: u64) -> Self {
        Self {
            slow_memory: false,
        }
    }*/

    /*pub fn system_default() -> Self {
        Self {
            slow_memory: false,
        }
    }*/

    pub fn from_limits(limits: limits_core::json::GenericGpuLimit) -> Self {
        Self {
            slow_memory: false,
            limits,
        }
    }

    pub fn from_json_and_limits(_other: GpuJson, _version: u64, limits: limits_core::json::GenericGpuLimit) -> Self {
        Self {
            slow_memory: false,
            limits,
        }
    }
}

impl Into<GpuJson> for Gpu {
    #[inline]
    fn into(self) -> GpuJson {
        GpuJson {
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            slow_memory: false,
        }
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), SettingError> {
        Ok(())
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), SettingError> {
        Ok(())
    }
}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        crate::api::GpuLimits {
            fast_ppt_limits: self.limits.fast_ppt.clone().map(|x| x.into()),
            slow_ppt_limits: self.limits.slow_ppt.clone().map(|x| x.into()),
            ppt_step: self.limits.ppt_step.unwrap_or(1_000_000),
            tdp_limits: self.limits.tdp.clone().map(|x| x.into()),
            tdp_boost_limits: self.limits.tdp_boost.clone().map(|x| x.into()),
            tdp_step: self.limits.tdp_step.unwrap_or(42),
            clock_min_limits: self.limits.clock_min.clone().map(|x| x.into()),
            clock_max_limits: self.limits.clock_max.clone().map(|x| x.into()),
            clock_step: self.limits.clock_step.unwrap_or(100),
            memory_control_capable: false,
        }
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.clone().into()
    }

    fn ppt(&mut self, _fast: Option<u64>, _slow: Option<u64>) {
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        (None, None)
    }

    fn clock_limits(&mut self, _limits: Option<MinMax<u64>>) {
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        None
    }

    fn slow_memory(&mut self) -> &mut bool {
        &mut self.slow_memory
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Generic
    }
}
