use std::convert::Into;

use limits_core::json::GenericGpuLimit;

use crate::settings::{MinMax, min_max_from_json};
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::TGpu;
use crate::persist::GpuJson;

#[derive(Debug, Clone)]
pub struct Gpu {
    pub slow_memory: bool,
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
    pub clock_limits: Option<MinMax<u64>>,
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
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            limits,
        }
    }

    pub fn from_json_and_limits(other: GpuJson, version: u64, limits: limits_core::json::GenericGpuLimit) -> Self {
        let clock_lims = if limits.clock_min.is_some() && limits.clock_max.is_some() {
            other.clock_limits.map(|x| min_max_from_json(x, version))
        } else {
            None
        };
        Self {
            slow_memory: false,
            fast_ppt: if limits.fast_ppt.is_some() {other.fast_ppt} else {None},
            slow_ppt: if limits.slow_ppt.is_some() {other.slow_ppt} else {None},
            clock_limits: clock_lims,
            limits,
        }
    }
}

impl Into<GpuJson> for Gpu {
    #[inline]
    fn into(self) -> GpuJson {
        GpuJson {
            fast_ppt: self.fast_ppt,
            slow_ppt: self.slow_ppt,
            clock_limits: self.clock_limits.map(|x| x.into()),
            slow_memory: false,
        }
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
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

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        if let Some(fast_lims) = &self.limits.fast_ppt {
            self.fast_ppt = fast.map(|x| x.clamp(fast_lims.min, fast_lims.max));
        }
        if let Some(slow_lims) = &self.limits.slow_ppt {
            self.slow_ppt = slow.map(|x| x.clamp(slow_lims.min, slow_lims.max));
        }
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        (self.fast_ppt, self.slow_ppt)
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        if let Some(clock_min) = &self.limits.clock_min {
            if let Some(clock_max) = &self.limits.clock_max {
                self.clock_limits = limits.map(|mut x| {
                    x.min = x.min.clamp(clock_min.min, clock_min.max);
                    x.max = x.max.clamp(clock_max.max, clock_max.max);
                    x
                });
            }
        }
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.clock_limits.as_ref()
    }

    fn slow_memory(&mut self) -> &mut bool {
        &mut self.slow_memory
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Generic
    }
}
