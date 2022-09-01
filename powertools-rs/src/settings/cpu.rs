use std::convert::Into;

use super::MinMax;
use super::{OnResume, OnSet, SettingError, SettingsRange};
use crate::persist::CpuJson;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub online: bool,
    pub clock_limits: Option<MinMax<u64>>,
    pub governor: String,
    index: usize,
}

const CPU_CLOCK_LIMITS_PATH: &str = "/sys/class/drm/card0/device/pp_od_clk_voltage";
const CPU_FORCE_LIMITS_PATH: &str = "/sys/class/drm/card0/device/power_dpm_force_performance_level";

const CPU_PRESENT_PATH: &str = "/sys/devices/system/cpu/present";

impl Cpu {
    #[inline]
    pub fn from_json(other: CpuJson, version: u64, i: usize) -> Self {
        match version {
            0 => Self {
                online: other.online,
                clock_limits: other.clock_limits.map(|x| MinMax::from_json(x, version)),
                governor: other.governor,
                index: i,
            },
            _ => Self {
                online: other.online,
                clock_limits: other.clock_limits.map(|x| MinMax::from_json(x, version)),
                governor: other.governor,
                index: i,
            },
        }
    }

    fn set_all(&self) -> Result<(), SettingError> {
        // set cpu online/offline
        if self.index != 0 { // cpu0 cannot be disabled
            let online_path = cpu_online_path(self.index);
            usdpl_back::api::files::write_single(&online_path, self.online as u8).map_err(|e| {
                SettingError {
                    msg: format!("Failed to write to `{}`: {}", &online_path, e),
                    setting: super::SettingVariant::Cpu,
                }
            })?;
        }
        // set clock limits
        if let Some(clock_limits) = &self.clock_limits {
            // set manual control
            usdpl_back::api::files::write_single(CPU_FORCE_LIMITS_PATH, "manual").map_err(|e| {
                SettingError {
                    msg: format!(
                        "Failed to write `manual` to `{}`: {}",
                        CPU_FORCE_LIMITS_PATH, e
                    ),
                    setting: super::SettingVariant::Cpu,
                }
            })?;
            // max clock
            let payload_max = format!("p {} 1 {}", self.index / 2, clock_limits.max);
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, &payload_max).map_err(
                |e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &payload_max, CPU_CLOCK_LIMITS_PATH, e
                    ),
                    setting: super::SettingVariant::Cpu,
                },
            )?;
            // min clock
            let payload_min = format!("p {} 0 {}", self.index / 2, clock_limits.min);
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, &payload_min).map_err(
                |e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &payload_min, CPU_CLOCK_LIMITS_PATH, e
                    ),
                    setting: super::SettingVariant::Cpu,
                },
            )?;
            // commit changes
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, "c").map_err(|e| {
                SettingError {
                    msg: format!("Failed to write `c` to `{}`: {}", CPU_CLOCK_LIMITS_PATH, e),
                    setting: super::SettingVariant::Cpu,
                }
            })?;
        } else {
            // disable manual clock limits
            usdpl_back::api::files::write_single(CPU_FORCE_LIMITS_PATH, "auto").map_err(|e| {
                SettingError {
                    msg: format!(
                        "Failed to write `auto` to `{}`: {}",
                        CPU_FORCE_LIMITS_PATH, e
                    ),
                    setting: super::SettingVariant::Cpu,
                }
            })?;
        }
        // set governor
        if self.index == 0 || self.online {
            let governor_path = cpu_governor_path(self.index);
            usdpl_back::api::files::write_single(&governor_path, &self.governor).map_err(|e| {
                SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &self.governor, &governor_path, e
                    ),
                    setting: super::SettingVariant::Cpu,
                }
            })?;
        }
        Ok(())
    }

    fn clamp_all(&mut self) {
        let min = Self::min();
        let max = Self::max();
        if let Some(clock_limits) = &mut self.clock_limits {
            let max_boost = max.clock_limits.as_ref().unwrap();
            let min_boost = min.clock_limits.as_ref().unwrap();
            clock_limits.min = clock_limits.min.clamp(min_boost.min, max_boost.min);
            clock_limits.max = clock_limits.max.clamp(min_boost.max, max_boost.max);
        }
    }

    fn from_sys(index: usize) -> Self {
        Self {
            online: usdpl_back::api::files::read_single(cpu_online_path(index)).unwrap_or(1u8) != 0,
            clock_limits: None,
            governor: usdpl_back::api::files::read_single(cpu_governor_path(index))
                .unwrap_or("schedutil".to_owned()),
            index: index,
        }
    }

    pub fn cpu_count() -> Option<usize> {
        let mut data: String = usdpl_back::api::files::read_single(CPU_PRESENT_PATH)
            .unwrap_or_else(|_| "0-7".to_string() /* Steam Deck's default */);
        if let Some(dash_index) = data.find('-') {
            let data = data.split_off(dash_index + 1);
            if let Ok(max_cpu) = data.parse::<usize>() {
                return Some(max_cpu + 1);
            }
        }
        log::warn!("Failed to parse CPU info from kernel, is Tux evil?");
        None
    }

    pub fn system_default() -> Vec<Self> {
        if let Some(max_cpu) = Self::cpu_count() {
            let mut cpus = Vec::with_capacity(max_cpu + 1);
            for i in 0..=max_cpu {
                cpus.push(Self::from_sys(i));
            }
            cpus
        } else {
            Vec::with_capacity(0)
        }
    }
}

impl Into<CpuJson> for Cpu {
    #[inline]
    fn into(self) -> CpuJson {
        CpuJson {
            online: self.online,
            clock_limits: self.clock_limits.map(|x| x.into()),
            governor: self.governor,
        }
    }
}

impl OnSet for Cpu {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Cpu {
    fn on_resume(&self) -> Result<(), SettingError> {
        self.set_all()
    }
}

impl SettingsRange for Cpu {
    #[inline]
    fn max() -> Self {
        Self {
            online: true,
            clock_limits: Some(MinMax {
                max: 3500,
                min: 3500,
            }),
            governor: "schedutil".to_owned(),
            index: usize::MAX,
        }
    }

    #[inline]
    fn min() -> Self {
        Self {
            online: false,
            clock_limits: Some(MinMax { max: 500, min: 1400 }),
            governor: "schedutil".to_owned(),
            index: 0,
        }
    }
}

#[inline]
fn cpu_online_path(index: usize) -> String {
    format!("/sys/devices/system/cpu/cpu{}/online", index)
}

#[inline]
fn cpu_governor_path(index: usize) -> String {
    format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        index
    )
}
