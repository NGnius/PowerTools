use std::convert::{Into, AsMut, AsRef};

use limits_core::json::GenericCpuLimit;

use crate::settings::{MinMax, min_max_from_json};
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{TCpus, TCpu};
use crate::persist::CpuJson;
use super::FromGenericCpuInfo;

const CPU_PRESENT_PATH: &str = "/sys/devices/system/cpu/present";
const CPU_SMT_PATH: &str = "/sys/devices/system/cpu/smt/control";

#[derive(Debug, Clone)]
pub struct Cpus<C: AsMut<Cpu> + AsRef<Cpu> + TCpu> {
    pub cpus: Vec<C>,
    pub smt: bool,
    pub smt_capable: bool,
}

impl<C: AsMut<Cpu> + AsRef<Cpu> + TCpu + OnSet> OnSet for Cpus<C> {
    fn on_set(&mut self) -> Result<(), SettingError> {
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
                })?;
            } else {
                usdpl_back::api::files::write_single(CPU_SMT_PATH, "off").map_err(|e| {
                    SettingError {
                        msg: format!(
                            "Failed to write `off` to `{}`: {}",
                            CPU_SMT_PATH, e
                        ),
                        setting: crate::settings::SettingVariant::Cpu,
                    }
                })?;
            }
        }
        for (i, cpu) in self.cpus.as_mut_slice().iter_mut().enumerate() {
            cpu.as_mut().state.do_set_online = self.smt || i % 2 == 0 || !self.smt_capable;
            cpu.on_set()?;
        }
        Ok(())
    }
}

impl<C: AsMut<Cpu> + AsRef<Cpu> + TCpu + OnResume> OnResume for Cpus<C> {
    fn on_resume(&self) -> Result<(), SettingError> {
        for cpu in &self.cpus {
            cpu.on_resume()?;
        }
        Ok(())
    }
}

impl<C: AsMut<Cpu> + AsRef<Cpu> + TCpu + FromGenericCpuInfo> Cpus<C> {
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

    pub fn from_limits(limits: limits_core::json::GenericCpuLimit) -> Self {
        let cpu_count = Self::cpu_count().unwrap_or(8);
        let (_, can_smt) = Self::system_smt_capabilities();
        let mut new_cpus = Vec::with_capacity(cpu_count);
        for i in 0..cpu_count {
            let new_cpu = C::from_limits(i, limits.clone());
            new_cpus.push(new_cpu);
        }
        Self {
            cpus: new_cpus,
            smt: true,
            smt_capable: can_smt,
        }
    }

    pub fn from_json_and_limits(mut other: Vec<CpuJson>, version: u64, limits: limits_core::json::GenericCpuLimit) -> Self {
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
            let new_cpu = C::from_json_and_limits(cpu, version, i, limits.clone());
            result.push(new_cpu);
        }
        if let Some(max_cpus) = max_cpus {
            if result.len() != max_cpus {
                let mut sys_cpus = Cpus::from_limits(limits.clone());
                for i in result.len()..sys_cpus.cpus.len() {
                    result.push(sys_cpus.cpus.remove(i));
                }
            }
        }
        Self {
            cpus: result,
            smt: smt_guess,
            smt_capable: can_smt,
        }
    }
}

impl<C: AsMut<Cpu> + AsRef<Cpu> + TCpu + OnResume + OnSet> TCpus for Cpus<C> {
    fn limits(&self) -> crate::api::CpusLimits {
        crate::api::CpusLimits {
            cpus: self.cpus.iter().map(|x| x.as_ref().limits()).collect(),
            count: self.cpus.len(),
            smt_capable: self.smt_capable,
        }
    }

    fn json(&self) -> Vec<crate::persist::CpuJson> {
        self.cpus.iter().map(|x| x.as_ref().to_owned().into()).collect()
    }

    fn cpus(&mut self) -> Vec<&mut dyn TCpu> {
        self.cpus.iter_mut().map(|x| x as &mut dyn TCpu).collect()
    }

    fn len(&self) -> usize {
        self.cpus.len()
    }

    fn smt(&mut self) -> &'_ mut bool {
        &mut self.smt
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Generic
    }
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub online: bool,
    pub governor: String,
    pub clock_limits: Option<MinMax<u64>>,
    limits: GenericCpuLimit,
    index: usize,
    state: crate::state::steam_deck::Cpu,
}

/*impl Cpu {
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }
}*/

impl AsRef<Cpu> for Cpu {
    #[inline]
    fn as_ref(&self) -> &Cpu {
        self
    }
}

impl AsMut<Cpu> for Cpu {
    #[inline]
    fn as_mut(&mut self) -> &mut Cpu {
        self
    }
}

impl FromGenericCpuInfo for Cpu {
    #[inline]
    fn from_limits(cpu_index: usize, limits: GenericCpuLimit) -> Self {
        Self {
            online: true,
            governor: "schedutil".to_owned(),
            clock_limits: None,
            limits,
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
        }
    }

    #[inline]
    fn from_json_and_limits(other: CpuJson, version: u64, i: usize, limits: GenericCpuLimit) -> Self {
        let clock_lims = if limits.clock_min.is_some() && limits.clock_max.is_some() {
            other.clock_limits.map(|x| min_max_from_json(x, version))
        } else {
            None
        };
        match version {
            0 => Self {
                online: other.online,
                governor: other.governor,
                clock_limits: clock_lims,
                limits,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
            },
            _ => Self {
                online: other.online,
                governor: other.governor,
                clock_limits: clock_lims,
                limits,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
            },
        }
    }
}

impl Cpu {
    fn set_all(&mut self) -> Result<(), SettingError> {
        // set cpu online/offline
        if self.index != 0 && self.state.do_set_online { // cpu0 cannot be disabled
            let online_path = cpu_online_path(self.index);
            usdpl_back::api::files::write_single(&online_path, self.online as u8).map_err(|e| {
                SettingError {
                    msg: format!("Failed to write to `{}`: {}", &online_path, e),
                    setting: crate::settings::SettingVariant::Cpu,
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
                    setting: crate::settings::SettingVariant::Cpu,
                }
            })?;
        }
        Ok(())
    }

    /*fn from_sys(cpu_index: usize) -> Self {
        Self {
            online: usdpl_back::api::files::read_single(cpu_online_path(cpu_index)).unwrap_or(1u8) != 0,
            governor: usdpl_back::api::files::read_single(cpu_governor_path(cpu_index))
                .unwrap_or("schedutil".to_owned()),
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
        }
    }*/

    fn governors(&self) -> Vec<String> {
        // NOTE: this eats errors
        let gov_str: String = match usdpl_back::api::files::read_single(cpu_available_governors_path(self.index)) {
            Ok(s) => s,
            Err((Some(e), None)) => {
                log::warn!("Error getting available CPU governors: {}", e);
                return vec![];
            },
            Err((None, Some(e))) => {
                log::warn!("Error getting available CPU governors: {}", e);
                return vec![];
            },
            Err(_) => return vec![],
        };
        gov_str.split(' ').map(|s| s.to_owned()).collect()
    }

    fn limits(&self) -> crate::api::CpuLimits {
        crate::api::CpuLimits {
            clock_min_limits: self.limits.clock_min.clone().map(|x| x.into()),
            clock_max_limits: self.limits.clock_max.clone().map(|x| x.into()),
            clock_step: self.limits.clock_step,
            governors: self.governors(),
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
        //self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Cpu {
    fn on_resume(&self) -> Result<(), SettingError> {
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
        if self.limits.clock_min.is_some() && self.limits.clock_max.is_some() {
            self.clock_limits = limits;
        }
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
