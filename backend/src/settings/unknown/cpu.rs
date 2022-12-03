use std::convert::Into;

use crate::settings::MinMax;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{TCpus, TCpu};
use crate::persist::CpuJson;

const CPU_PRESENT_PATH: &str = "/sys/devices/system/cpu/present";
const CPU_SMT_PATH: &str = "/sys/devices/system/cpu/smt/control";

#[derive(Debug, Clone)]
pub struct Cpus {
    pub cpus: Vec<Cpu>,
    pub smt: bool,
    pub smt_capable: bool,
}

impl OnSet for Cpus {
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
            cpu.state.do_set_online = self.smt || i % 2 == 0;
            cpu.on_set()?;
        }
        Ok(())
    }
}

impl OnResume for Cpus {
    fn on_resume(&self) -> Result<(), SettingError> {
        for cpu in &self.cpus {
            cpu.on_resume()?;
        }
        Ok(())
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
        if let Some(max_cpu) = Self::cpu_count() {
            let mut sys_cpus = Vec::with_capacity(max_cpu);
            for i in 0..max_cpu {
                sys_cpus.push(Cpu::from_sys(i));
            }
            let (smt_status, can_smt) = Self::system_smt_capabilities();
            Self {
                cpus: sys_cpus,
                smt: smt_status,
                smt_capable: can_smt,
            }
        } else {
            Self {
                cpus: vec![],
                smt: false,
                smt_capable: false,
            }
        }
    }

    #[inline]
    pub fn from_json(mut other: Vec<CpuJson>, version: u64) -> Self {
        let (_, can_smt) = Self::system_smt_capabilities();
        let mut result = Vec::with_capacity(other.len());
        let max_cpus = Self::cpu_count();
        for (i, cpu) in other.drain(..).enumerate() {
            // prevent having more CPUs than available
            if let Some(max_cpus) = max_cpus {
                if i == max_cpus {
                    break;
                }
            }
            result.push(Cpu::from_json(cpu, version, i));
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
            smt: true,
            smt_capable: can_smt,
        }
    }
}

impl TCpus for Cpus {
    fn limits(&self) -> crate::api::CpusLimits {
        crate::api::CpusLimits {
            cpus: self.cpus.iter().map(|x| x.limits()).collect(),
            count: self.cpus.len(),
            smt_capable: self.smt_capable,
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

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Unknown
    }
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub online: bool,
    pub governor: String,
    index: usize,
    state: crate::state::steam_deck::Cpu,
}


impl Cpu {
    #[inline]
    pub fn from_json(other: CpuJson, version: u64, i: usize) -> Self {
        match version {
            0 => Self {
                online: other.online,
                governor: other.governor,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
            },
            _ => Self {
                online: other.online,
                governor: other.governor,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
            },
        }
    }

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

    fn from_sys(cpu_index: usize) -> Self {
        Self {
            online: usdpl_back::api::files::read_single(cpu_online_path(cpu_index)).unwrap_or(1u8) != 0,
            governor: usdpl_back::api::files::read_single(cpu_governor_path(cpu_index))
                .unwrap_or("schedutil".to_owned()),
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
        }
    }

    fn limits(&self) -> crate::api::CpuLimits {
        crate::api::CpuLimits {
            clock_min_limits: None,
            clock_max_limits: None,
            clock_step: 100,
            governors: vec![], // TODO
        }
    }
}

impl Into<CpuJson> for Cpu {
    #[inline]
    fn into(self) -> CpuJson {
        CpuJson {
            online: self.online,
            clock_limits: None,
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

    fn clock_limits(&mut self, _limits: Option<MinMax<u64>>) {
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        None
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
