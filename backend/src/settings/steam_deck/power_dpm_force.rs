//! Be very careful when using this.
//! This influences Steam Deck CPU and GPU driver behaviour,
//! so familiarize yourself with those before messing with this functionality.
//! Refer to https://docs.kernel.org/5.19/gpu/amdgpu/thermal.html for kernel stuff.

use std::sync::atomic::{AtomicU64, Ordering};

use sysfuss::{BasicEntityPath, SysEntityAttributesExt, SysAttribute};

use crate::settings::SettingError;

const DEFAULT_BITS: u64 = 0;

/// Global usage tracker for the sysfs file by the same name
pub static POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT: PDFPLManager =
    PDFPLManager(AtomicU64::new(DEFAULT_BITS));

pub struct PDFPLManager(AtomicU64);

//const OVERRIDE_BIT: usize = 0;
const GPU_BIT: usize = 1;
const CPU_BITS_START: usize = 2;

//const DPM_FORCE_LIMITS_PATH: &str = "/sys/class/drm/card0/device/power_dpm_force_performance_level";
pub const DPM_FORCE_LIMITS_ATTRIBUTE: &str = "device/power_dpm_force_performance_level";

impl PDFPLManager {
    #[inline]
    fn get(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }

    #[inline]
    fn set(&self, val: u64) {
        self.0.store(val, Ordering::SeqCst);
    }

    #[inline]
    fn set_bit(&self, val: bool, bit: usize) {
        let bitmask: u64 = !(1 << bit);
        let val: u64 = (val as u64) << bit;
        let new_val = (self.get() & bitmask) | val;
        self.set(new_val);
    }

    pub fn set_gpu(&self, manual: bool) {
        self.set_bit(manual, GPU_BIT);
    }

    pub fn set_cpu(&self, manual: bool, cpu: usize) {
        self.set_bit(manual, CPU_BITS_START + cpu);
    }

    pub fn needs_manual(&self) -> bool {
        self.get() != 0
    }

    pub fn reset(&self) {
        self.set(DEFAULT_BITS);
    }

    pub fn enforce_level(&self, entity: &BasicEntityPath) -> Result<(), Vec<SettingError>> {
        let needs = self.needs_manual();
        let mut errors = Vec::new();
        let path = DPM_FORCE_LIMITS_ATTRIBUTE.path(entity);
        let mode: String = entity.attribute(DPM_FORCE_LIMITS_ATTRIBUTE.to_owned())
            .map_err(|e| {
                vec![SettingError {
                    msg: format!("Failed to read `{}`: {}", path.display(), e),
                    setting: crate::settings::SettingVariant::General,
                }]
            })?;
        if mode != "manual" && needs {
            log::info!("Setting `{}` to manual", path.display());
            // set manual control
            entity.set(DPM_FORCE_LIMITS_ATTRIBUTE.to_owned(), "manual")
                .map_err(|e| {
                    errors.push(SettingError {
                        msg: format!("Failed to write `manual` to `{}`: {}", path.display(), e),
                        setting: crate::settings::SettingVariant::General,
                    })
                })
                .unwrap_or(());
        } else if mode != "auto" && !needs {
            log::info!("Setting `{}` to auto", path.display());
            // unset manual control
            entity.set(DPM_FORCE_LIMITS_ATTRIBUTE.to_owned(), "auto")
                .map_err(|e| {
                    errors.push(SettingError {
                        msg: format!("Failed to write `auto` to `{}`: {}", path.display(), e),
                        setting: crate::settings::SettingVariant::General,
                    })
                })
                .unwrap_or(());
        }
        if let Ok(mode_now) =
            entity.attribute::<String, _>(DPM_FORCE_LIMITS_ATTRIBUTE.to_owned())
        {
            log::debug!("Mode for `{}` is now `{}`", path.display(), mode_now);
        } else {
            log::debug!("Error getting new mode for debugging purposes");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
