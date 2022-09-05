use std::convert::Into;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::{Battery, Cpu, Gpu};
use super::{OnResume, OnSet, SettingError};
use crate::persist::{CpuJson, SettingsJson};
use crate::utility::unwrap_lock;

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
        // remove settings file when persistence is turned off, to prevent it from be loaded next time.
        if !self.persistent && self.path.exists() {
            std::fs::remove_file(&self.path).map_err(|e| SettingError {
                msg: format!("Failed to delete `{}`: {}", self.path.display(), e),
                setting: SettingVariant::General,
            })?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub general: Arc<Mutex<General>>,
    pub cpus: Arc<Mutex<Vec<Cpu>>>,
    pub gpu: Arc<Mutex<Gpu>>,
    pub battery: Arc<Mutex<Battery>>,
}

impl OnSet for Settings {
    fn on_set(&mut self) -> Result<(), SettingError> {
        unwrap_lock(self.battery.lock(), "battery").on_set()?;
        {
            // cpu lock scope
            let mut cpu_lock = unwrap_lock(self.cpus.lock(), "cpu");
            for cpu in cpu_lock.iter_mut() {
                cpu.on_set()?;
            }
        }
        unwrap_lock(self.gpu.lock(), "gpu").on_set()?;
        unwrap_lock(self.general.lock(), "general").on_set()?;
        Ok(())
    }
}

impl Settings {
    #[inline]
    pub fn from_json(other: SettingsJson, json_path: PathBuf) -> Self {
        match other.version {
            0 => Self {
                general: Arc::new(Mutex::new(General {
                    persistent: other.persistent,
                    path: json_path,
                    name: other.name,
                })),
                cpus: Arc::new(Mutex::new(Self::convert_cpus(other.cpus, other.version))),
                gpu: Arc::new(Mutex::new(Gpu::from_json(other.gpu, other.version))),
                battery: Arc::new(Mutex::new(Battery::from_json(other.battery, other.version))),
            },
            _ => Self {
                general: Arc::new(Mutex::new(General {
                    persistent: other.persistent,
                    path: json_path,
                    name: other.name,
                })),
                cpus: Arc::new(Mutex::new(Self::convert_cpus(other.cpus, other.version))),
                gpu: Arc::new(Mutex::new(Gpu::from_json(other.gpu, other.version))),
                battery: Arc::new(Mutex::new(Battery::from_json(other.battery, other.version))),
            },
        }
    }

    fn convert_cpus(mut cpus: Vec<CpuJson>, version: u64) -> Vec<Cpu> {
        let mut result = Vec::with_capacity(cpus.len());
        for (i, cpu) in cpus.drain(..).enumerate() {
            result.push(Cpu::from_json(cpu, version, i));
        }
        result
    }

    pub fn system_default(json_path: PathBuf) -> Self {
        Self {
            general: Arc::new(Mutex::new(General {
                persistent: false,
                path: json_path,
                name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
            })),
            cpus: Arc::new(Mutex::new(Cpu::system_default())),
            gpu: Arc::new(Mutex::new(Gpu::system_default())),
            battery: Arc::new(Mutex::new(Battery::system_default())),
        }
    }
    
    pub fn load_file(&self, filename: PathBuf, name: String) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(filename);
        let mut general_lock = unwrap_lock(self.general.lock(), "general");
        if json_path.exists() {
            let settings_json = SettingsJson::open(&json_path).map_err(|e| SettingError {
                msg: e.to_string(),
                setting: SettingVariant::General,
            })?;
            let new_cpus = Self::convert_cpus(settings_json.cpus, settings_json.version);
            let new_gpu = Gpu::from_json(settings_json.gpu, settings_json.version);
            let new_battery = Battery::from_json(settings_json.battery, settings_json.version);
            {
                let mut cpu_lock = unwrap_lock(self.cpus.lock(), "cpu");
                *cpu_lock = new_cpus; // TODO does this overwrite the contents of the lock as expected?
            }
            {
                let mut gpu_lock = unwrap_lock(self.gpu.lock(), "gpu");
                *gpu_lock = new_gpu;
            }
            {
                let mut battery_lock = unwrap_lock(self.battery.lock(), "battery");
                *battery_lock = new_battery;
            }
            general_lock.persistent = true;
            general_lock.name = settings_json.name;
        } else {
            general_lock.persistent = false;
            general_lock.name = name;
        }
        general_lock.path = json_path;
        Ok(general_lock.persistent)
    }
}

impl OnResume for Settings {
    fn on_resume(&self) -> Result<(), SettingError> {
        unwrap_lock(self.battery.lock(), "battery").on_resume()?;
        {
            let mut cpu_lock = unwrap_lock(self.cpus.lock(), "cpu");
            for cpu in cpu_lock.iter_mut() {
                cpu.on_resume()?;
            }
        }
        unwrap_lock(self.gpu.lock(), "gpu").on_resume()?;
        Ok(())
    }
}

impl Into<SettingsJson> for Settings {
    #[inline]
    fn into(self) -> SettingsJson {
        log::debug!("Locking settings to convert into json");
        let gen_lock = unwrap_lock(self.general.lock(), "general");
        log::debug!("Got general lock");
        let cpu_lock = unwrap_lock(self.cpus.lock(), "cpu");
        log::debug!("Got cpus lock");
        let gpu_lock = unwrap_lock(self.gpu.lock(), "gpu");
        log::debug!("Got gpu lock");
        let batt_lock = unwrap_lock(self.battery.lock(), "battery");
        log::debug!("Got battery lock");
        SettingsJson {
            version: LATEST_VERSION,
            name: gen_lock.name.clone(),
            persistent: gen_lock.persistent,
            cpus: cpu_lock
                .clone()
                .drain(..)
                .map(|cpu| cpu.into())
                .collect(),
            gpu: gpu_lock.clone().into(),
            battery: batt_lock.clone().into(),
        }
    }
}
