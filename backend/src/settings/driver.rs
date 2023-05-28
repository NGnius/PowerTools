use super::{auto_detect0, General, SettingError, TBattery, TCpus, TGeneral, TGpu};
use crate::persist::{DriverJson, SettingsJson};

pub struct Driver {
    pub general: Box<dyn TGeneral>,
    pub cpus: Box<dyn TCpus>,
    pub gpu: Box<dyn TGpu>,
    pub battery: Box<dyn TBattery>,
}

impl Driver {
    pub fn init(
        settings: SettingsJson,
        json_path: std::path::PathBuf,
    ) -> Result<Self, SettingError> {
        Ok(match settings.version {
            0 => Self::version0(settings, json_path)?,
            _ => Self {
                general: Box::new(General {
                    persistent: settings.persistent,
                    path: json_path,
                    name: settings.name,
                    driver: DriverJson::SteamDeck,
                    events: settings.events.unwrap_or_default(),
                }),
                cpus: Box::new(super::steam_deck::Cpus::from_json(
                    settings.cpus,
                    settings.version,
                )),
                gpu: Box::new(super::steam_deck::Gpu::from_json(
                    settings.gpu,
                    settings.version,
                )),
                battery: Box::new(super::steam_deck::Battery::from_json(
                    settings.battery,
                    settings.version,
                )),
            },
        })
    }

    fn version0(
        settings: SettingsJson,
        json_path: std::path::PathBuf,
    ) -> Result<Self, SettingError> {
        let name = settings.name.clone();
        if let Some(provider) = &settings.provider {
            match provider {
                DriverJson::SteamDeck => Ok(Self {
                    general: Box::new(General {
                        persistent: settings.persistent,
                        path: json_path,
                        name: settings.name,
                        driver: DriverJson::SteamDeck,
                        events: settings.events.unwrap_or_default(),
                    }),
                    cpus: Box::new(super::steam_deck::Cpus::from_json(
                        settings.cpus,
                        settings.version,
                    )),
                    gpu: Box::new(super::steam_deck::Gpu::from_json(
                        settings.gpu,
                        settings.version,
                    )),
                    battery: Box::new(super::steam_deck::Battery::from_json(
                        settings.battery,
                        settings.version,
                    )),
                }),
                // There's nothing special about SteamDeckAdvance, it just appears different
                DriverJson::SteamDeckAdvance => Ok(Self {
                    general: Box::new(General {
                        persistent: settings.persistent,
                        path: json_path,
                        name: settings.name,
                        driver: DriverJson::SteamDeckAdvance,
                        events: settings.events.unwrap_or_default(),
                    }),
                    cpus: Box::new(super::steam_deck::Cpus::from_json(
                        settings.cpus,
                        settings.version,
                    )),
                    gpu: Box::new(super::steam_deck::Gpu::from_json(
                        settings.gpu,
                        settings.version,
                    )),
                    battery: Box::new(super::steam_deck::Battery::from_json(
                        settings.battery,
                        settings.version,
                    )),
                }),
                DriverJson::Generic | DriverJson::GenericAMD => {
                    Ok(super::detect::auto_detect0(Some(settings), json_path, name))
                }
                DriverJson::Unknown => {
                    Ok(super::detect::auto_detect0(Some(settings), json_path, name))
                }
                DriverJson::AutoDetect => {
                    Ok(super::detect::auto_detect0(Some(settings), json_path, name))
                }
            }
        } else {
            Ok(super::detect::auto_detect0(Some(settings), json_path, name))
        }
    }

    pub fn system_default(json_path: std::path::PathBuf, name: String) -> Self {
        auto_detect0(None, json_path, name)
    }
}

// sshhhh, this function isn't here ;)
#[inline]
pub fn maybe_do_button() {
    match super::auto_detect_provider() {
        DriverJson::SteamDeck | DriverJson::SteamDeckAdvance => {
            crate::settings::steam_deck::flash_led();
        }
        DriverJson::Generic | DriverJson::GenericAMD => {
            log::warn!("You need to come up with something fun on generic")
        }
        DriverJson::Unknown => log::warn!("Can't do button activities on unknown platform"),
        DriverJson::AutoDetect => log::warn!("WTF, why is auto_detect detecting AutoDetect???"),
    }
}
