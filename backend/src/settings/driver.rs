use crate::persist::{DriverJson, SettingsJson};
use super::{TGeneral, TCpus, TGpu, TBattery, SettingError, General};

/// Device detection logic
fn auto_detect() -> DriverJson {
    let lscpu: String = match usdpl_back::api::files::read_single("/proc/cpuinfo") {
        Ok(s) => s,
        Err(_) => return DriverJson::Unknown,
    };
    log::debug!("Read from /proc/cpuinfo:\n{}", lscpu);
    let os_info: String = match usdpl_back::api::files::read_single("/etc/os-release") {
        Ok(s) => s,
        Err(_) => return DriverJson::Unknown,
    };
    log::debug!("Read from /etc/os-release:\n{}", os_info);
    if let Some(_) = lscpu.find("model name\t: AMD Custom APU 0405\n") {
        // definitely a Steam Deck, check if it's overclocked
        // TODO: this auto-detect doesn't work
        // look for a file instead?
        let max_freq: u64 = match usdpl_back::api::files::read_single("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
            Ok(u) => u,
            Err(_) => return DriverJson::SteamDeck,
        };
        if max_freq == 2800000 { // default clock speed
            DriverJson::SteamDeck
        } else {
            DriverJson::SteamDeckAdvance
        }
    } else {
        DriverJson::Unknown
    }
}

#[inline]
pub fn auto_detect_loud() -> DriverJson {
    let provider = auto_detect();
    log::info!("Detected device automatically, compatible driver: {:?}", provider);
    provider
}

pub struct Driver {
    pub general: Box<dyn TGeneral>,
    pub cpus: Box<dyn TCpus>,
    pub gpu: Box<dyn TGpu>,
    pub battery: Box<dyn TBattery>,
}

impl Driver {
    pub fn init(settings: SettingsJson, json_path: std::path::PathBuf) -> Result<Self, SettingError> {
        Ok(match settings.version {
            0 => Self::version0(settings, json_path)?,
            _ => Self {
                general: Box::new(General {
                    persistent: settings.persistent,
                    path: json_path,
                    name: settings.name,
                    driver: DriverJson::SteamDeck,
                }),
                cpus: Box::new(super::steam_deck::Cpus::from_json(settings.cpus, settings.version)),
                gpu: Box::new(super::steam_deck::Gpu::from_json(settings.gpu, settings.version)),
                battery: Box::new(super::steam_deck::Battery::from_json(settings.battery, settings.version)),
            },
        })
    }

    fn version0(settings: SettingsJson, json_path: std::path::PathBuf) -> Result<Self, SettingError> {
        let provider = settings.provider.unwrap_or_else(auto_detect);
        match provider {
            DriverJson::SteamDeck => Ok(Self {
                general: Box::new(General {
                    persistent: settings.persistent,
                    path: json_path,
                    name: settings.name,
                    driver: DriverJson::SteamDeck,
                }),
                cpus: Box::new(super::steam_deck::Cpus::from_json(settings.cpus, settings.version)),
                gpu: Box::new(super::steam_deck::Gpu::from_json(settings.gpu, settings.version)),
                battery: Box::new(super::steam_deck::Battery::from_json(settings.battery, settings.version)),
            }),
            DriverJson::SteamDeckAdvance => Ok(Self {
                general: Box::new(General {
                    persistent: settings.persistent,
                    path: json_path,
                    name: settings.name,
                    driver: DriverJson::SteamDeckAdvance,
                }),
                cpus: Box::new(super::steam_deck_adv::Cpus::from_json(settings.cpus, settings.version)),
                gpu: Box::new(super::steam_deck_adv::Gpu::from_json(settings.gpu, settings.version)),
                battery: Box::new(super::steam_deck::Battery::from_json(settings.battery, settings.version)),
            }),
            DriverJson::Unknown => Ok(Self {
                general: Box::new(General {
                    persistent: settings.persistent,
                    path: json_path,
                    name: settings.name,
                    driver: DriverJson::Unknown,
                }),
                cpus: Box::new(super::unknown::Cpus::from_json(settings.cpus, settings.version)),
                gpu: Box::new(super::unknown::Gpu::from_json(settings.gpu, settings.version)),
                battery: Box::new(super::unknown::Battery),
            }),
        }
    }

    pub fn system_default(json_path: std::path::PathBuf) -> Self {
        let provider = auto_detect();
        match provider {
            DriverJson::SteamDeck => Self {
                general: Box::new(General {
                    persistent: false,
                    path: json_path,
                    name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
                    driver: DriverJson::SteamDeck,
                }),
                cpus: Box::new(super::steam_deck::Cpus::system_default()),
                gpu: Box::new(super::steam_deck::Gpu::system_default()),
                battery: Box::new(super::steam_deck::Battery::system_default()),
            },
            DriverJson::SteamDeckAdvance => Self {
                general: Box::new(General {
                    persistent: false,
                    path: json_path,
                    name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
                    driver: DriverJson::SteamDeck,
                }),
                cpus: Box::new(super::steam_deck_adv::Cpus::system_default()),
                gpu: Box::new(super::steam_deck_adv::Gpu::system_default()),
                battery: Box::new(super::steam_deck::Battery::system_default()),
            },
            DriverJson::Unknown => Self {
                general: Box::new(General {
                    persistent: false,
                    path: json_path,
                    name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
                    driver: DriverJson::Unknown,
                }),
                cpus: Box::new(super::unknown::Cpus::system_default()),
                gpu: Box::new(super::unknown::Gpu::system_default()),
                battery: Box::new(super::unknown::Battery),
            }
        }
    }
}

// static battery calls

#[inline]
pub fn read_current_now() -> Result<Option<u64>, SettingError> {
    match auto_detect() {
        DriverJson::SteamDeck => super::steam_deck::Battery::read_current_now().map(|x| Some(x)),
        DriverJson::SteamDeckAdvance => super::steam_deck::Battery::read_current_now().map(|x| Some(x)),
        DriverJson::Unknown => Ok(None),
    }
}

#[inline]
pub fn read_charge_now() -> Result<Option<f64>, SettingError> {
    match auto_detect() {
        DriverJson::SteamDeck => super::steam_deck::Battery::read_charge_now().map(|x| Some(x)),
        DriverJson::SteamDeckAdvance => super::steam_deck::Battery::read_charge_now().map(|x| Some(x)),
        DriverJson::Unknown => Ok(None),
    }
}

#[inline]
pub fn read_charge_full() -> Result<Option<f64>, SettingError> {
    match auto_detect() {
        DriverJson::SteamDeck => super::steam_deck::Battery::read_charge_full().map(|x| Some(x)),
        DriverJson::SteamDeckAdvance => super::steam_deck::Battery::read_charge_full().map(|x| Some(x)),
        DriverJson::Unknown => Ok(None),
    }
}

#[inline]
pub fn read_charge_design() -> Result<Option<f64>, SettingError> {
    match auto_detect() {
        DriverJson::SteamDeck => super::steam_deck::Battery::read_charge_design().map(|x| Some(x)),
        DriverJson::SteamDeckAdvance => super::steam_deck::Battery::read_charge_design().map(|x| Some(x)),
        DriverJson::Unknown => Ok(None),
    }
}

#[inline]
pub fn maybe_do_button() {
    match auto_detect() {
        DriverJson::SteamDeck | DriverJson::SteamDeckAdvance => {
            let period = std::time::Duration::from_millis(500);
            for _ in 0..10 {
                if let Err(e) = crate::settings::steam_deck::set_led(false, true, false) {
                    log::error!("Thing err: {}", e);
                }
                std::thread::sleep(period);
                if let Err(e) = crate::settings::steam_deck::set_led(false, false, false) {
                    log::error!("Thing err: {}", e);
                };
                std::thread::sleep(period);
            }
        },
        DriverJson::Unknown => log::warn!("Can't do button activities on unknown platform"),
    }
}
