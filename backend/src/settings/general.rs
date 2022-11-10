use std::convert::Into;
use std::path::PathBuf;
//use std::sync::{Arc, Mutex};

use super::{Battery, Cpu, Gpu};
use super::{OnResume, OnSet, SettingError};
use crate::persist::{CpuJson, SettingsJson};
//use crate::utility::unwrap_lock;

const LATEST_VERSION: u64 = 0;

#[derive(Debug, Clone, Copy)]
pub enum SettingVariant {
    Battery,
    Cpu,
    Gpu,
    General,
}

impl std::fmt::Display for SettingVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Battery => write!(f, "Battery"),
            Self::Cpu => write!(f, "CPU"),
            Self::Gpu => write!(f, "GPU"),
            Self::General => write!(f, "General"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct General {
    pub persistent: bool,
    pub path: PathBuf,
    pub name: String,
}

impl OnSet for General {
    fn on_set(&mut self) -> Result<(), SettingError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub general: General,
    pub cpus: Vec<Cpu>,
    pub gpu: Gpu,
    pub battery: Battery,
}

impl OnSet for Settings {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.battery.on_set()?;
        for cpu in self.cpus.iter_mut() {
            cpu.on_set()?;
        }
        self.gpu.on_set()?;
        self.general.on_set()?;
        Ok(())
    }
}

impl Settings {
    #[inline]
    pub fn from_json(other: SettingsJson, json_path: PathBuf) -> Self {
        match other.version {
            0 => Self {
                general: General {
                    persistent: other.persistent,
                    path: json_path,
                    name: other.name,
                },
                cpus: Self::convert_cpus(other.cpus, other.version),
                gpu: Gpu::from_json(other.gpu, other.version),
                battery: Battery::from_json(other.battery, other.version),
            },
            _ => Self {
                general: General {
                    persistent: other.persistent,
                    path: json_path,
                    name: other.name,
                },
                cpus: Self::convert_cpus(other.cpus, other.version),
                gpu: Gpu::from_json(other.gpu, other.version),
                battery: Battery::from_json(other.battery, other.version),
            },
        }
    }

    fn convert_cpus(mut cpus: Vec<CpuJson>, version: u64) -> Vec<Cpu> {
        let mut result = Vec::with_capacity(cpus.len());
        let max_cpus = Cpu::cpu_count();
        for (i, cpu) in cpus.drain(..).enumerate() {
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
                let mut sys_cpus = Cpu::system_default();
                for i in result.len()..sys_cpus.len() {
                    result.push(sys_cpus.remove(i));
                }
            }
        }
        result
    }

    pub fn system_default(json_path: PathBuf) -> Self {
        Self {
            general: General {
                persistent: false,
                path: json_path,
                name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
            },
            cpus: Cpu::system_default(),
            gpu: Gpu::system_default(),
            battery: Battery::system_default(),
        }
    }

    pub fn load_system_default(&mut self) {
        self.cpus = Cpu::system_default();
        self.gpu = Gpu::system_default();
        self.battery = Battery::system_default();
    }
    
    pub fn load_file(&mut self, filename: PathBuf, name: String, system_defaults: bool) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(filename);
        //let mut general_lock = unwrap_lock(self.general.lock(), "general");
        if json_path.exists() {
            let settings_json = SettingsJson::open(&json_path).map_err(|e| SettingError {
                msg: e.to_string(),
                setting: SettingVariant::General,
            })?;
            if !settings_json.persistent {
                log::warn!("Loaded persistent config `{}` ({}) with persistent=false", &settings_json.name, json_path.display());
                self.general.persistent = false;
                self.general.name = name;
            } else {
                self.cpus = Self::convert_cpus(settings_json.cpus, settings_json.version);
                self.gpu = Gpu::from_json(settings_json.gpu, settings_json.version);
                self.battery = Battery::from_json(settings_json.battery, settings_json.version);
                self.general.persistent = true;
                self.general.name = settings_json.name;
            }
        } else {
            if system_defaults {
                self.load_system_default();
            }
            self.general.persistent = false;
            self.general.name = name;
        }
        self.general.path = json_path;
        Ok(self.general.persistent)
    }
}

impl OnResume for Settings {
    fn on_resume(&self) -> Result<(), SettingError> {
        log::debug!("Applying settings for on_resume");
        self.battery.on_resume()?;
        log::debug!("Resumed battery");
        for cpu in self.cpus.iter() {
            cpu.on_resume()?;
        }
        log::debug!("Resumed CPUs");
        self.gpu.on_resume()?;
        log::debug!("Resumed GPU");
        Ok(())
    }
}

impl Into<SettingsJson> for Settings {
    #[inline]
    fn into(self) -> SettingsJson {
        log::debug!("Converting into json");
        SettingsJson {
            version: LATEST_VERSION,
            name: self.general.name.clone(),
            persistent: self.general.persistent,
            cpus: self.cpus
                .clone()
                .drain(..)
                .map(|cpu| cpu.into())
                .collect(),
            gpu: self.gpu.clone().into(),
            battery: self.battery.clone().into(),
        }
    }
}
