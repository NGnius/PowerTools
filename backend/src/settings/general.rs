use std::path::PathBuf;
//use std::sync::{Arc, Mutex};

//use super::{Battery, Cpus, Gpu};
use super::{OnResume, OnSet, SettingError};
use super::{TGeneral, TGpu, TCpus, TBattery};
use crate::persist::SettingsJson;
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
    pub driver: crate::persist::DriverJson,
}

impl OnSet for General {
    fn on_set(&mut self) -> Result<(), SettingError> {
        Ok(())
    }
}

impl OnResume for General {
    fn on_resume(&self) -> Result<(), SettingError> {
        Ok(())
    }
}

impl TGeneral for General {
    fn limits(&self) -> crate::api::GeneralLimits {
        crate::api::GeneralLimits {  }
    }

    fn get_persistent(&self) -> bool {
        self.persistent
    }

    fn persistent(&mut self) -> &'_ mut bool {
        &mut self.persistent
    }

    fn get_path(&self) -> &'_ std::path::Path {
        &self.path
    }

    fn path(&mut self, path: std::path::PathBuf) {
        self.path = path;
    }

    fn get_name(&self) -> &'_ str {
        &self.name
    }

    fn name(&mut self, name: String) {
        self.name = name;
    }

    fn provider(&self) -> crate::persist::DriverJson {
        self.driver.clone()
    }
}

#[derive(Debug)]
pub struct Settings {
    pub general: Box<dyn TGeneral>,
    pub cpus: Box<dyn TCpus>,
    pub gpu: Box<dyn TGpu>,
    pub battery: Box<dyn TBattery>,
}

impl OnSet for Settings {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.battery.on_set()?;
        self.cpus.on_set()?;
        self.gpu.on_set()?;
        self.general.on_set()?;
        Ok(())
    }
}

impl Settings {
    #[inline]
    pub fn from_json(other: SettingsJson, json_path: PathBuf) -> Self {
        match super::Driver::init(other, json_path.clone()) {
            Ok(x) => {
                log::info!("Loaded settings with drivers general:{:?},cpus:{:?},gpu:{:?},battery:{:?}", x.general.provider(), x.cpus.provider(), x.gpu.provider(), x.battery.provider());
                Self {
                    general: x.general,
                    cpus: x.cpus,
                    gpu: x.gpu,
                    battery: x.battery,
                }
            },
            Err(e) => {
                log::error!("Driver init error: {}", e);
                Self::system_default(json_path)
            }
        }
    }

    pub fn system_default(json_path: PathBuf) -> Self {
        let driver = super::Driver::system_default(json_path);
        Self {
            general: driver.general,
            cpus: driver.cpus,
            gpu: driver.gpu,
            battery: driver.battery,
        }
    }

    pub fn load_system_default(&mut self) {
        let driver = super::Driver::system_default(self.general.get_path().to_owned());
        self.cpus = driver.cpus;
        self.gpu = driver.gpu;
        self.battery = driver.battery;
    }

    pub fn load_file(&mut self, filename: PathBuf, name: String, system_defaults: bool) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(filename);
        if json_path.exists() {
            let settings_json = SettingsJson::open(&json_path).map_err(|e| SettingError {
                msg: e.to_string(),
                setting: SettingVariant::General,
            })?;
            if !settings_json.persistent {
                log::warn!("Loaded persistent config `{}` ({}) with persistent=false", &settings_json.name, json_path.display());
                *self.general.persistent() = false;
                self.general.name(name);
            } else {
                match super::Driver::init(settings_json, json_path.clone()) {
                    Ok(x) => {
                        log::info!("Loaded settings with drivers general:{:?},cpus:{:?},gpu:{:?},battery:{:?}", x.general.provider(), x.cpus.provider(), x.gpu.provider(), x.battery.provider());
                        self.general = x.general;
                        self.cpus = x.cpus;
                        self.gpu = x.gpu;
                        self.battery = x.battery;
                    },
                    Err(e) => {
                        log::error!("Driver init error: {}", e);
                        self.general.name(name);
                        *self.general.persistent() = false;
                        self.general.path(json_path);
                        return Err(e);
                    }
                };
            }
        } else {
            if system_defaults {
                self.load_system_default();
            }
            *self.general.persistent() = false;
            self.general.name(name);
        }
        self.general.path(json_path);
        Ok(*self.general.persistent())
    }
    
    /*
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
                *self.general.persistent() = false;
                self.general.name(name);
            } else {
                self.cpus = Box::new(super::steam_deck::Cpus::from_json(settings_json.cpus, settings_json.version));
                self.gpu = Box::new(super::steam_deck::Gpu::from_json(settings_json.gpu, settings_json.version));
                self.battery = Box::new(super::steam_deck::Battery::from_json(settings_json.battery, settings_json.version));
                *self.general.persistent() = true;
                self.general.name(settings_json.name);
            }
        } else {
            if system_defaults {
                self.load_system_default();
            }
            *self.general.persistent() = false;
            self.general.name(name);
        }
        self.general.path(json_path);
        Ok(*self.general.persistent())
    }*/

    pub fn json(&self) -> SettingsJson {
        SettingsJson {
            version: LATEST_VERSION,
            name: self.general.get_name().to_owned(),
            persistent: self.general.get_persistent(),
            cpus: self.cpus.json(),
            gpu: self.gpu.json(),
            battery: self.battery.json(),
            provider: Some(self.general.provider()),
        }
    }
}

impl OnResume for Settings {
    fn on_resume(&self) -> Result<(), SettingError> {
        log::debug!("Applying settings for on_resume");
        self.battery.on_resume()?;
        log::debug!("Resumed battery");
        self.cpus.on_resume()?;
        log::debug!("Resumed CPUs");
        self.gpu.on_resume()?;
        log::debug!("Resumed GPU");
        Ok(())
    }
}

/*impl Into<SettingsJson> for Settings {
    #[inline]
    fn into(self) -> SettingsJson {
        log::debug!("Converting into json");
        SettingsJson {
            version: LATEST_VERSION,
            name: self.general.get_name().to_owned(),
            persistent: self.general.get_persistent(),
            cpus: self.cpus.json(),
            gpu: self.gpu.json(),
            battery: self.battery.json(),
            provider: Some(self.general.provider()),
        }
    }
}*/
