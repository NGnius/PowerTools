use std::path::PathBuf;
//use std::sync::{Arc, Mutex};

//use super::{Battery, Cpus, Gpu};
use super::{OnResume, OnSet, SettingError};
use super::{TBattery, TCpus, TGeneral, TGpu};
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
    pub events: crate::persist::OnEventJson,
}

impl OnSet for General {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        if let Some(event) = &self.events.on_set {
            if !event.is_empty() {
                std::process::Command::new("/bin/bash")
                    .args(&["-c", event])
                    .spawn()
                    .map_err(|e| {
                        vec![SettingError {
                            msg: format!("on_set event command error: {}", e),
                            setting: SettingVariant::General,
                        }]
                    })?;
            }
        }
        Ok(())
    }
}

impl OnResume for General {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        if let Some(event) = &self.events.on_resume {
            if !event.is_empty() {
                std::process::Command::new("/bin/bash")
                    .args(&["-c", event])
                    .spawn()
                    .map_err(|e| {
                        vec![SettingError {
                            msg: format!("on_resume event command error: {}", e),
                            setting: SettingVariant::General,
                        }]
                    })?;
            }
        }
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for General {}

impl TGeneral for General {
    fn limits(&self) -> crate::api::GeneralLimits {
        crate::api::GeneralLimits {}
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

    fn on_event(&self) -> &crate::persist::OnEventJson {
        &self.events
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
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        log::debug!("Applying settings for on_set");
        self.general
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set general");
        self.battery
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set battery");
        self.cpus
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set CPUs");
        self.gpu
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set GPU");

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Settings {
    #[inline]
    pub fn from_json(other: SettingsJson, json_path: PathBuf) -> Self {
        let name_bup = other.name.clone();
        match super::Driver::init(other, json_path.clone()) {
            Ok(x) => {
                log::info!(
                    "Loaded settings with drivers general:{:?},cpus:{:?},gpu:{:?},battery:{:?}",
                    x.general.provider(),
                    x.cpus.provider(),
                    x.gpu.provider(),
                    x.battery.provider()
                );
                Self {
                    general: x.general,
                    cpus: x.cpus,
                    gpu: x.gpu,
                    battery: x.battery,
                }
            }
            Err(e) => {
                log::error!("Driver init error: {}", e);
                Self::system_default(json_path, name_bup)
            }
        }
    }

    pub fn system_default(json_path: PathBuf, name: String) -> Self {
        let driver = super::Driver::system_default(json_path, name);
        Self {
            general: driver.general,
            cpus: driver.cpus,
            gpu: driver.gpu,
            battery: driver.battery,
        }
    }

    pub fn load_system_default(&mut self, name: String) {
        let driver = super::Driver::system_default(self.general.get_path().to_owned(), name);
        self.cpus = driver.cpus;
        self.gpu = driver.gpu;
        self.battery = driver.battery;
        self.general = driver.general;
    }

    pub fn load_file(
        &mut self,
        filename: PathBuf,
        name: String,
        system_defaults: bool,
    ) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(&filename);
        if json_path.exists() {
            let settings_json = SettingsJson::open(&json_path).map_err(|e| SettingError {
                msg: e.to_string(),
                setting: SettingVariant::General,
            })?;
            if !settings_json.persistent {
                log::warn!(
                    "Loaded persistent config `{}` ({}) with persistent=false",
                    &settings_json.name,
                    json_path.display()
                );
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
                    }
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
                self.load_system_default(name);
            } else {
                self.general.name(name);
            }
            *self.general.persistent() = false;
        }
        self.general.path(filename);
        if let Some(event) = &self.general.on_event().on_load {
            if !event.is_empty() {
                std::process::Command::new("/bin/bash")
                    .args(&["-c", event])
                    .spawn()
                    .map_err(|e| SettingError {
                        msg: format!("on_save event command error: {}", e),
                        setting: SettingVariant::General,
                    })?;
            }
        }
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
            events: Some(self.general.on_event().clone()),
        }
    }
}

impl OnResume for Settings {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        log::debug!("Applying settings for on_resume");
        self.general
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed general");
        self.battery
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed battery");
        self.cpus
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed CPUs");
        self.gpu
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed GPU");

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl crate::settings::OnPowerEvent for Settings {
    fn on_power_event(&mut self, new_mode: super::PowerMode) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        self.general
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.battery
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.cpus
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.gpu
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
