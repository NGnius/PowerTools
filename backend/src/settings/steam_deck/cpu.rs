use std::convert::Into;

use crate::api::RangeLimit;
use crate::settings::{MinMax, min_max_from_json};
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{TCpus, TCpu};
use crate::persist::CpuJson;
use super::oc_limits::{OverclockLimits, CpusLimits, CpuLimits};

const CPU_PRESENT_PATH: &str = "/sys/devices/system/cpu/present";
const CPU_SMT_PATH: &str = "/sys/devices/system/cpu/smt/control";

#[derive(Debug, Clone)]
pub struct Cpus {
    pub cpus: Vec<Cpu>,
    pub smt: bool,
    pub smt_capable: bool,
    pub(super) limits: CpusLimits,
    driver_mode: crate::persist::DriverJson,
}

impl OnSet for Cpus {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        if self.smt_capable {
            // toggle SMT
            if self.smt {
                usdpl_back::api::files::write_single(CPU_SMT_PATH, "on").map_err(|e| {
                    SettingError {
                        msg: format!(
                            "Failed to write `on` to `{}`: {}",
                            CPU_SMT_PATH, e
                        ),
                        setting: crate::settings::SettingVariant::Cpu,
                    }
                }).unwrap_or_else(|e| errors.push(e));
            } else {
                usdpl_back::api::files::write_single(CPU_SMT_PATH, "off").map_err(|e| {
                    SettingError {
                        msg: format!(
                            "Failed to write `off` to `{}`: {}",
                            CPU_SMT_PATH, e
                        ),
                        setting: crate::settings::SettingVariant::Cpu,
                    }
                }).unwrap_or_else(|e| errors.push(e));
            }
        }
        for (i, cpu) in self.cpus.as_mut_slice().iter_mut().enumerate() {
            cpu.state.do_set_online = self.smt || i % 2 == 0;
            cpu.on_set().unwrap_or_else(|mut e| errors.append(&mut e));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl OnResume for Cpus {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        for cpu in &self.cpus {
            cpu.on_resume().unwrap_or_else(|mut e| errors.append(&mut e));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Cpus {
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

    fn system_smt_capabilities() -> (bool, bool) {
        match usdpl_back::api::files::read_single::<_, String, _>(CPU_SMT_PATH) {
            Ok(val) => (val.trim().to_lowercase() == "on", true),
            Err(_) => (false, false)
        }
    }

    pub fn system_default() -> Self {
        let (oc_limits, is_default) = OverclockLimits::load_or_default();
        let oc_limits = oc_limits.cpus;
        let driver = if is_default { crate::persist::DriverJson::SteamDeck } else { crate::persist::DriverJson::SteamDeckAdvance };
        if let Some(max_cpu) = Self::cpu_count() {
            let mut sys_cpus = Vec::with_capacity(max_cpu);
            for i in 0..max_cpu {
                sys_cpus.push(Cpu::system_default(i, oc_limits.cpus.get(i).map(|x| x.to_owned()).unwrap_or_default()));
            }
            let (_, can_smt) = Self::system_smt_capabilities();
            Self {
                cpus: sys_cpus,
                smt: true,
                smt_capable: can_smt,
                limits: oc_limits,
                driver_mode: driver,
            }
        } else {
            Self {
                cpus: vec![],
                smt: false,
                smt_capable: false,
                limits: oc_limits,
                driver_mode: driver,
            }
        }
    }

    #[inline]
    pub fn from_json(mut other: Vec<CpuJson>, version: u64) -> Self {
        let (oc_limits, is_default) = OverclockLimits::load_or_default();
        let oc_limits = oc_limits.cpus;
        let driver = if is_default { crate::persist::DriverJson::SteamDeck } else { crate::persist::DriverJson::SteamDeckAdvance };
        let (_, can_smt) = Self::system_smt_capabilities();
        let mut result = Vec::with_capacity(other.len());
        let max_cpus = Self::cpu_count();
        let smt_guess = crate::settings::util::guess_smt(&other) && can_smt;
        for (i, cpu) in other.drain(..).enumerate() {
            // prevent having more CPUs than available
            if let Some(max_cpus) = max_cpus {
                if i == max_cpus {
                    break;
                }
            }
            let new_cpu = Cpu::from_json(cpu, version, i, oc_limits.cpus.get(i).map(|x| x.to_owned()).unwrap_or_default());
            result.push(new_cpu);
        }
        if let Some(max_cpus) = max_cpus {
            if result.len() != max_cpus {
                let mut sys_cpus = Cpus::system_default();
                for i in result.len()..sys_cpus.cpus.len() {
                    result.push(sys_cpus.cpus.remove(i));
                }
            }
        }
        Self {
            cpus: result,
            smt: smt_guess,
            smt_capable: can_smt,
            limits: oc_limits,
            driver_mode: driver,
        }
    }
}

impl TCpus for Cpus {
    fn limits(&self) -> crate::api::CpusLimits {
        crate::api::CpusLimits {
            cpus: self.cpus.iter().map(|x| x.limits()).collect(),
            count: self.cpus.len(),
            smt_capable: self.smt_capable,
            governors: if self.limits.global_governors {
                self.cpus.iter()
                    .next()
                    .map(|x| x.governors())
                    .unwrap_or_else(|| Vec::with_capacity(0))
            } else { Vec::with_capacity(0) },
        }
    }

    fn json(&self) -> Vec<crate::persist::CpuJson> {
        self.cpus.iter().map(|x| x.to_owned().into()).collect()
    }

    fn cpus(&mut self) -> Vec<&mut dyn TCpu> {
        self.cpus.iter_mut().map(|x| x as &mut dyn TCpu).collect()
    }

    fn len(&self) -> usize {
        self.cpus.len()
    }

    fn smt(&mut self) -> &'_ mut bool {
        log::debug!("CPU driver thinks SMT is {}", self.smt);
        &mut self.smt
    }

    fn provider(&self) -> crate::persist::DriverJson {
        self.driver_mode.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub online: bool,
    pub clock_limits: Option<MinMax<u64>>,
    pub governor: String,
    limits: CpuLimits,
    index: usize,
    state: crate::state::steam_deck::Cpu,
}

const CPU_CLOCK_LIMITS_PATH: &str = "/sys/class/drm/card0/device/pp_od_clk_voltage";
const CPU_FORCE_LIMITS_PATH: &str = "/sys/class/drm/card0/device/power_dpm_force_performance_level";

impl Cpu {
    #[inline]
    fn from_json(other: CpuJson, version: u64, i: usize, oc_limits: CpuLimits) -> Self {
        match version {
            0 => Self {
                online: other.online,
                clock_limits: other.clock_limits.map(|x| min_max_from_json(x, version)),
                governor: other.governor,
                limits: oc_limits,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
            },
            _ => Self {
                online: other.online,
                clock_limits: other.clock_limits.map(|x| min_max_from_json(x, version)),
                governor: other.governor,
                limits: oc_limits,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
            },
        }
    }

    fn set_force_performance_related(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        // set clock limits
        log::debug!("Setting {} to manual", CPU_FORCE_LIMITS_PATH);
        let mode: String = usdpl_back::api::files::read_single(CPU_FORCE_LIMITS_PATH.to_owned()).unwrap();
        if mode != "manual" {
            // set manual control
            usdpl_back::api::files::write_single(CPU_FORCE_LIMITS_PATH, "manual").map_err(|e| {
                vec![SettingError {
                    msg: format!(
                        "Failed to write `manual` to `{}`: {}",
                        CPU_FORCE_LIMITS_PATH, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                }]
            })?;
        }
        if let Some(clock_limits) = &self.clock_limits {
            log::debug!("Setting CPU {} (min, max) clockspeed to ({}, {})", self.index, clock_limits.min, clock_limits.max);
            self.state.clock_limits_set = true;
            // max clock
            let payload_max = format!("p {} 1 {}\n", self.index / 2, clock_limits.max);
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, &payload_max).map_err(
                |e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &payload_max, CPU_CLOCK_LIMITS_PATH, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                },
            ).unwrap_or_else(|e| errors.push(e));
            // min clock
            let valid_min = if clock_limits.min < self.limits.clock_min.min {self.limits.clock_min.min} else {clock_limits.min};
            let payload_min = format!("p {} 0 {}\n", self.index / 2, valid_min);
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, &payload_min).map_err(
                |e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &payload_min, CPU_CLOCK_LIMITS_PATH, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                },
            ).unwrap_or_else(|e| errors.push(e));
        } else if self.state.clock_limits_set || (self.state.is_resuming && !self.limits.skip_resume_reclock) {
            self.state.clock_limits_set = false;
            // disable manual clock limits
            log::debug!("Setting CPU {} to default clockspeed", self.index);
            // max clock
            let payload_max = format!("p {} 1 {}\n", self.index / 2, self.limits.clock_max.max);
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, &payload_max).map_err(
                |e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &payload_max, CPU_CLOCK_LIMITS_PATH, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                },
            ).unwrap_or_else(|e| errors.push(e));
            // min clock
            let payload_min = format!("p {} 0 {}\n", self.index / 2, self.limits.clock_min.min);
            usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, &payload_min).map_err(
                |e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &payload_min, CPU_CLOCK_LIMITS_PATH, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                },
            ).unwrap_or_else(|e| errors.push(e));
        }
        // commit changes
        usdpl_back::api::files::write_single(CPU_CLOCK_LIMITS_PATH, "c\n")
            .unwrap_or_else(|e| {
                errors.push(SettingError {
                    msg: format!("Failed to write `c` to `{}`: {}", CPU_CLOCK_LIMITS_PATH, e),
                    setting: crate::settings::SettingVariant::Cpu,
                });
            });

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        // set cpu online/offline
        if self.index != 0 && self.state.do_set_online { // cpu0 cannot be disabled
            let online_path = cpu_online_path(self.index);
            usdpl_back::api::files::write_single(&online_path, self.online as u8).map_err(|e| {
                SettingError {
                    msg: format!("Failed to write to `{}`: {}", &online_path, e),
                    setting: crate::settings::SettingVariant::Cpu,
                }
            }).unwrap_or_else(|e| errors.push(e));
        }

        self.set_force_performance_related()
            .unwrap_or_else(|mut e| errors.append(&mut e));

        // set governor
        if self.index == 0 || self.online {
            let governor_path = cpu_governor_path(self.index);
            usdpl_back::api::files::write_single(&governor_path, &self.governor).map_err(|e| {
                SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &self.governor, &governor_path, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                }
            }).unwrap_or_else(|e| errors.push(e));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn clamp_all(&mut self) {
        if let Some(clock_limits) = &mut self.clock_limits {
            clock_limits.min = clock_limits.min.clamp(self.limits.clock_min.min, self.limits.clock_min.max);
            clock_limits.max = clock_limits.max.clamp(self.limits.clock_max.min, self.limits.clock_max.max);
        }
    }

    /*fn from_sys(cpu_index: usize, oc_limits: CpuLimits) -> Self {
        Self {
            online: usdpl_back::api::files::read_single(cpu_online_path(cpu_index)).unwrap_or(1u8) != 0,
            clock_limits: None,
            governor: usdpl_back::api::files::read_single(cpu_governor_path(cpu_index))
                .unwrap_or("schedutil".to_owned()),
            limits: oc_limits,
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
        }
    }*/

    fn system_default(cpu_index: usize, oc_limits: CpuLimits) -> Self {
        Self {
            online: true,
            clock_limits: None,
            governor: "schedutil".to_owned(),
            limits: oc_limits,
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
        }
    }

    fn limits(&self) -> crate::api::CpuLimits {
        crate::api::CpuLimits {
            clock_min_limits: Some(RangeLimit {
                min: self.limits.clock_max.min, // allows min to be set by max (it's weird, blame the kernel)
                max: self.limits.clock_min.max
            }),
            clock_max_limits: Some(RangeLimit {
                min: self.limits.clock_max.min,
                max: self.limits.clock_max.max
            }),
            clock_step: self.limits.clock_step,
            governors: self.governors(),
        }
    }

    fn governors(&self) -> Vec<String> {
        // NOTE: this eats errors
        let gov_str: String = match usdpl_back::api::files::read_single(cpu_available_governors_path(self.index)) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Error getting available CPU governors: {}", e);
                return vec![];
            }
        };
        gov_str.split(' ').map(|s| s.to_owned()).collect()
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
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Cpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut copy = self.clone();
        copy.state.is_resuming = true;
        copy.set_all()
    }
}

impl TCpu for Cpu {
    fn online(&mut self) -> &mut bool {
        &mut self.online
    }

    fn governor(&mut self, governor: String) {
        self.governor = governor;
    }

    fn get_governor(&self) -> &'_ str {
        &self.governor
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.clock_limits = limits;
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.clock_limits.as_ref()
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


#[inline]
fn cpu_available_governors_path(index: usize) -> String {
    format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors",
        index
    )
}
