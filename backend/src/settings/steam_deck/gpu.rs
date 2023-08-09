use std::convert::Into;

use sysfuss::{BasicEntityPath, HwMonPath, SysEntity, capability::attributes, SysEntityAttributesExt, SysAttribute};

use super::oc_limits::{GpuLimits, OverclockLimits};
use super::POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT;
use crate::api::RangeLimit;
use crate::persist::GpuJson;
use crate::settings::TGpu;
use crate::settings::{min_max_from_json, MinMax};
use crate::settings::{OnResume, OnSet, SettingError};

// usually in /sys/class/hwmon/hwmon4/<attribute>
const SLOW_PPT_ATTRIBUTE: sysfuss::HwMonAttribute = sysfuss::HwMonAttribute::custom("power1_cap");
const FAST_PPT_ATTRIBUTE: sysfuss::HwMonAttribute = sysfuss::HwMonAttribute::custom("power2_cap");

#[derive(Debug, Clone)]
pub struct Gpu {
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
    pub clock_limits: Option<MinMax<u64>>,
    pub slow_memory: bool,
    limits: GpuLimits,
    state: crate::state::steam_deck::Gpu,
    driver_mode: crate::persist::DriverJson,
    sysfs_card: BasicEntityPath,
    sysfs_hwmon: HwMonPath
}

// same as CPU
//const GPU_CLOCK_LIMITS_PATH: &str = "/sys/class/drm/card0/device/pp_od_clk_voltage";
//const GPU_MEMORY_DOWNCLOCK_PATH: &str = "/sys/class/drm/card0/device/pp_dpm_fclk";

const GPU_CLOCK_LIMITS_ATTRIBUTE: &str = "device/pp_od_clk_voltage";
const GPU_MEMORY_DOWNCLOCK_ATTRIBUTE: &str = "device/pp_dpm_fclk";

const CARD_EXTENSIONS: &[&'static str] = &[
    GPU_CLOCK_LIMITS_ATTRIBUTE,
    GPU_MEMORY_DOWNCLOCK_ATTRIBUTE,
    super::DPM_FORCE_LIMITS_ATTRIBUTE,
];

enum ClockType {
    Min = 0,
    Max = 1,
}

impl Gpu {
    #[inline]
    pub fn from_json(other: GpuJson, version: u64) -> Self {
        let (oc_limits, is_default) = OverclockLimits::load_or_default();
        let driver = if is_default {
            crate::persist::DriverJson::SteamDeck
        } else {
            crate::persist::DriverJson::SteamDeckAdvance
        };
        match version {
            0 => Self {
                fast_ppt: other.fast_ppt,
                slow_ppt: other.slow_ppt,
                clock_limits: other.clock_limits.map(|x| min_max_from_json(x, version)),
                slow_memory: other.slow_memory,
                limits: oc_limits.gpu,
                state: crate::state::steam_deck::Gpu::default(),
                driver_mode: driver,
                sysfs_card: Self::find_card_sysfs(other.root.clone()),
                sysfs_hwmon: Self::find_hwmon_sysfs(other.root),
            },
            _ => Self {
                fast_ppt: other.fast_ppt,
                slow_ppt: other.slow_ppt,
                clock_limits: other.clock_limits.map(|x| min_max_from_json(x, version)),
                slow_memory: other.slow_memory,
                limits: oc_limits.gpu,
                state: crate::state::steam_deck::Gpu::default(),
                driver_mode: driver,
                sysfs_card: Self::find_card_sysfs(other.root.clone()),
                sysfs_hwmon: Self::find_hwmon_sysfs(other.root),
            },
        }
    }

    fn find_card_sysfs(root: Option<impl AsRef<std::path::Path>>) -> BasicEntityPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        match root.class("drm", attributes(crate::settings::util::CARD_NEEDS.into_iter().map(|s| s.to_string()))) {
            Ok(iter) => {
                let card = iter
                    .filter(|ent| if let Ok(name) = ent.name() { name.starts_with("card")} else { false })
                    .filter(|ent| super::util::card_also_has(ent, CARD_EXTENSIONS))
                    .next()
                    .unwrap_or_else(|| {
                        log::error!("Failed to find SteamDeck gpu drm in sysfs (no results), using naive fallback");
                        BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
                    });
                log::info!("Found SteamDeck gpu drm in sysfs: {}", card.as_ref().display());
                card
            },
            Err(e) => {
                log::error!("Failed to find SteamDeck gpu drm in sysfs ({}), using naive fallback", e);
                BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
            }
        }
    }

    fn find_hwmon_sysfs(root: Option<impl AsRef<std::path::Path>>) -> HwMonPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        let hwmon = root.hwmon_by_name(super::util::GPU_HWMON_NAME).unwrap_or_else(|e| {
            log::error!("Failed to find SteamDeck gpu hwmon in sysfs ({}), using naive fallback", e);
            root.hwmon_by_index(4)
        });
        log::info!("Found SteamDeck gpu hwmon {} in sysfs: {}", super::util::GPU_HWMON_NAME, hwmon.as_ref().display());
        hwmon
    }

    fn set_clock_limit(&self, speed: u64, mode: ClockType) -> Result<(), SettingError> {
        let payload = format!("s {} {}\n", mode as u8, speed);
        let path = GPU_CLOCK_LIMITS_ATTRIBUTE.path(&self.sysfs_card);
        self.sysfs_card.set(GPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), &payload).map_err(|e| {
            SettingError {
                msg: format!("Failed to write `{}` to `{}`: {}", &payload, path.display(), e),
                setting: crate::settings::SettingVariant::Gpu,
            }
        })
    }

    fn set_confirm(&self) -> Result<(), SettingError> {
        let path = GPU_CLOCK_LIMITS_ATTRIBUTE.path(&self.sysfs_card);
        self.sysfs_card.set(GPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), "c\n").map_err(|e| {
            SettingError {
                msg: format!("Failed to write `c` to `{}`: {}", path.display(), e),
                setting: crate::settings::SettingVariant::Gpu,
            }
        })
    }

    fn set_clocks(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        if let Some(clock_limits) = &self.clock_limits {
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(true);
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs_card)?;
            // set clock limits
            self.state.clock_limits_set = true;
            // max clock
            if let Some(max) = clock_limits.max {
                self.set_clock_limit(max, ClockType::Max).unwrap_or_else(|e| errors.push(e));
            }
            // min clock
            if let Some(min) = clock_limits.min {
                self.set_clock_limit(min, ClockType::Min).unwrap_or_else(|e| errors.push(e));
            }

            self.set_confirm().unwrap_or_else(|e| errors.push(e));
        } else if self.state.clock_limits_set
            || (self.state.is_resuming && !self.limits.skip_resume_reclock)
            || POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual()
        {
            self.state.clock_limits_set = false;
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(self.slow_memory);
            if POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual() {
                POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs_card)?;
                // disable manual clock limits
                // max clock
                self.set_clock_limit(self.limits.clock_max.max, ClockType::Max)
                    .unwrap_or_else(|e| errors.push(e));
                // min clock
                self.set_clock_limit(self.limits.clock_min.min, ClockType::Min)
                    .unwrap_or_else(|e| errors.push(e));

                self.set_confirm().unwrap_or_else(|e| errors.push(e));
            } else {
                POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT
                    .enforce_level(&self.sysfs_card)
                    .unwrap_or_else(|mut e| errors.append(&mut e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn set_slow_memory(&self, slow: bool) -> Result<(), SettingError> {
        let path = GPU_MEMORY_DOWNCLOCK_ATTRIBUTE.path(&self.sysfs_card);
        self.sysfs_card.set(GPU_MEMORY_DOWNCLOCK_ATTRIBUTE.to_owned(), slow as u8).map_err(|e| {
            SettingError {
                msg: format!("Failed to write to `{}`: {}", path.display(), e),
                setting: crate::settings::SettingVariant::Gpu,
            }
        })
    }

    fn set_force_performance_related(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        // enable/disable downclock of GPU memory (to 400Mhz?)
        if self.slow_memory {
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(true);
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT
                .enforce_level(&self.sysfs_card)
                .unwrap_or_else(|mut e| errors.append(&mut e));
            self.set_slow_memory(self.slow_memory).unwrap_or_else(|e| errors.push(e));
        } else if POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual() {
            self.set_slow_memory(self.slow_memory).unwrap_or_else(|e| errors.push(e));
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(self.clock_limits.is_some());
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT
                .enforce_level(&self.sysfs_card)
                .unwrap_or_else(|mut e| errors.append(&mut e));
        }
        self.set_clocks()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        // commit changes (if no errors have already occured)
        if errors.is_empty() {
            if self.slow_memory || self.clock_limits.is_some() {
                self.set_confirm().map_err(|e| {
                    errors.push(e);
                    errors
                })
            } else {
                Ok(())
            }
        } else {
            Err(errors)
        }
    }

    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        // set fast PPT
        if let Some(fast_ppt) = &self.fast_ppt {
            self.state.fast_ppt_set = true;
            self.sysfs_hwmon.set(FAST_PPT_ATTRIBUTE, fast_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        fast_ppt, FAST_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        } else if self.state.fast_ppt_set {
            self.state.fast_ppt_set = false;
            let fast_ppt = self.limits.fast_ppt_default;
            self.sysfs_hwmon.set(FAST_PPT_ATTRIBUTE, fast_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        fast_ppt, FAST_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        }
        // set slow PPT
        if let Some(slow_ppt) = &self.slow_ppt {
            self.state.slow_ppt_set = true;
            self.sysfs_hwmon.set(SLOW_PPT_ATTRIBUTE, slow_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        slow_ppt, SLOW_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        } else if self.state.slow_ppt_set {
            self.state.slow_ppt_set = false;
            let slow_ppt = self.limits.slow_ppt_default;
            self.sysfs_hwmon.set(SLOW_PPT_ATTRIBUTE, slow_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        slow_ppt, SLOW_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        }
        self.set_force_performance_related()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn clamp_all(&mut self) {
        if let Some(fast_ppt) = &mut self.fast_ppt {
            *fast_ppt = (*fast_ppt).clamp(self.limits.fast_ppt.min, self.limits.fast_ppt.max);
        }
        if let Some(slow_ppt) = &mut self.slow_ppt {
            *slow_ppt = (*slow_ppt).clamp(self.limits.slow_ppt.min, self.limits.slow_ppt.max);
        }
        if let Some(clock_limits) = &mut self.clock_limits {
            if let Some(min) = clock_limits.min {
                clock_limits.min =
                    Some(min.clamp(self.limits.clock_min.min, self.limits.clock_min.max));
            }
            if let Some(max) = clock_limits.max {
                clock_limits.max =
                    Some(max.clamp(self.limits.clock_max.min, self.limits.clock_max.max));
            }
        }
    }

    pub fn system_default() -> Self {
        let (oc_limits, is_default) = OverclockLimits::load_or_default();
        Self {
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            slow_memory: false,
            limits: oc_limits.gpu,
            state: crate::state::steam_deck::Gpu::default(),
            driver_mode: if is_default {
                crate::persist::DriverJson::SteamDeck
            } else {
                crate::persist::DriverJson::SteamDeckAdvance
            },
            sysfs_card: Self::find_card_sysfs(None::<&'static str>),
            sysfs_hwmon: Self::find_hwmon_sysfs(None::<&'static str>),
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
            slow_memory: self.slow_memory,
            root: self.sysfs_card.root().or(self.sysfs_hwmon.root()).and_then(|p| p.as_ref().to_str().map(|r| r.to_owned()))
        }
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut copy = self.clone();
        copy.state.is_resuming = true;
        copy.set_all()
    }
}

impl crate::settings::OnPowerEvent for Gpu {}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        crate::api::GpuLimits {
            fast_ppt_limits: Some(RangeLimit {
                min: self.limits.fast_ppt.min / self.limits.ppt_divisor,
                max: self.limits.fast_ppt.max / self.limits.ppt_divisor,
            }),
            slow_ppt_limits: Some(RangeLimit {
                min: self.limits.slow_ppt.min / self.limits.ppt_divisor,
                max: self.limits.slow_ppt.max / self.limits.ppt_divisor,
            }),
            ppt_step: self.limits.ppt_step,
            tdp_limits: None,
            tdp_boost_limits: None,
            tdp_step: 42,
            clock_min_limits: Some(RangeLimit {
                min: self.limits.clock_min.min,
                max: self.limits.clock_min.max,
            }),
            clock_max_limits: Some(RangeLimit {
                min: self.limits.clock_max.min,
                max: self.limits.clock_max.max,
            }),
            clock_step: self.limits.clock_step,
            memory_control_capable: true,
        }
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.clone().into()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        self.fast_ppt = fast.map(|x| x * self.limits.ppt_divisor);
        self.slow_ppt = slow.map(|x| x * self.limits.ppt_divisor);
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        (
            self.fast_ppt.map(|x| x / self.limits.ppt_divisor),
            self.slow_ppt.map(|x| x / self.limits.ppt_divisor),
        )
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.clock_limits = limits;
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.clock_limits.as_ref()
    }

    fn slow_memory(&mut self) -> &mut bool {
        &mut self.slow_memory
    }

    fn provider(&self) -> crate::persist::DriverJson {
        self.driver_mode.clone()
    }
}
