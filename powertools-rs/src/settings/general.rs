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
        {
            // general lock scope
            let gen_lock = unwrap_lock(self.general.lock(), "general");
            if !gen_lock.persistent && gen_lock.path.exists() {
                std::fs::remove_file(&gen_lock.path).map_err(|e| SettingError {
                    msg: format!("Failed to delete `{}`: {}", gen_lock.path.display(), e),
                    setting: SettingVariant::General,
                })?;
            }
        }
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
                name: "".to_owned(),
            })),
            cpus: Arc::new(Mutex::new(Cpu::system_default())),
            gpu: Arc::new(Mutex::new(Gpu::system_default())),
            battery: Arc::new(Mutex::new(Battery::system_default())),
        }
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
        SettingsJson {
            version: LATEST_VERSION,
            name: unwrap_lock(self.general.lock(), "general").name.clone(),
            persistent: unwrap_lock(self.general.lock(), "general").persistent,
            cpus: unwrap_lock(self.cpus.lock(), "cpu")
                .clone()
                .drain(..)
                .map(|cpu| cpu.into())
                .collect(),
            gpu: unwrap_lock(self.gpu.lock(), "gpu").clone().into(),
            battery: unwrap_lock(self.battery.lock(), "battery").clone().into(),
        }
    }
}
