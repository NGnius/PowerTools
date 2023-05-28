use std::fs::File;

use regex::RegexBuilder;

use limits_core::json::{BatteryLimit, CpuLimit, GpuLimit, Limits};

use crate::persist::{DriverJson, SettingsJson};
use crate::settings::{Driver, General, TBattery, TCpus, TGeneral, TGpu};

fn get_limits() -> limits_core::json::Base {
    let limits_path = super::utility::limits_path();
    match File::open(&limits_path) {
        Ok(f) => match serde_json::from_reader(f) {
            Ok(lim) => lim,
            Err(e) => {
                log::warn!(
                    "Failed to parse limits file `{}`, cannot use for auto_detect: {}",
                    limits_path.display(),
                    e
                );
                limits_core::json::Base::default()
            }
        },
        Err(e) => {
            log::warn!(
                "Failed to open limits file `{}`: {}",
                limits_path.display(),
                e
            );
            super::limits_worker::get_limits_cached()
        }
    }
}

#[inline]
pub fn auto_detect_provider() -> DriverJson {
    let provider = auto_detect0(
        None,
        crate::utility::settings_dir().join("autodetect.json"),
        "".to_owned(),
    )
    .battery
    .provider();
    //log::info!("Detected device automatically, compatible driver: {:?}", provider);
    provider
}

/// Device detection logic
pub fn auto_detect0(
    settings_opt: Option<SettingsJson>,
    json_path: std::path::PathBuf,
    name: String,
) -> Driver {
    let mut builder = DriverBuilder::new(json_path, name);

    let cpu_info: String = usdpl_back::api::files::read_single("/proc/cpuinfo").unwrap_or_default();
    log::debug!("Read from /proc/cpuinfo:\n{}", cpu_info);
    let os_info: String =
        usdpl_back::api::files::read_single("/etc/os-release").unwrap_or_default();
    log::debug!("Read from /etc/os-release:\n{}", os_info);
    let dmi_info: String = std::process::Command::new("dmidecode")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default();
    log::debug!("Read dmidecode:\n{}", dmi_info);

    let limits = get_limits();

    // build driver based on limits conditions
    for conf in limits.configs {
        let conditions = conf.conditions;
        let mut matches = true;
        if conditions.is_empty() {
            matches = !builder.is_complete();
        } else {
            if let Some(dmi) = &conditions.dmi {
                let pattern = RegexBuilder::new(dmi)
                    .multi_line(true)
                    .build()
                    .expect("Invalid DMI regex");
                matches &= pattern.is_match(&dmi_info);
            }
            if let Some(cpuinfo) = &conditions.cpuinfo {
                let pattern = RegexBuilder::new(cpuinfo)
                    .multi_line(true)
                    .build()
                    .expect("Invalid CPU regex");
                matches &= pattern.is_match(&cpu_info);
            }
            if let Some(os) = &conditions.os {
                let pattern = RegexBuilder::new(os)
                    .multi_line(true)
                    .build()
                    .expect("Invalid OS regex");
                matches &= pattern.is_match(&os_info);
            }
            if let Some(cmd) = &conditions.command {
                match std::process::Command::new("bash")
                    .args(["-c", cmd])
                    .status()
                {
                    Ok(status) => matches &= status.code().map(|c| c == 0).unwrap_or(false),
                    Err(e) => log::warn!("Ignoring bash limits error: {}", e),
                }
            }
            if let Some(file_exists) = &conditions.file_exists {
                let exists = std::path::Path::new(file_exists).exists();
                matches &= exists;
            }
        }
        if matches {
            if let Some(settings) = &settings_opt {
                *builder.general.persistent() = true;
                builder.general.name(settings.name.clone());
                for limit in conf.limits {
                    match limit {
                        Limits::Cpu(cpus) => {
                            let cpu_driver: Box<dyn TCpus> = match cpus {
                                CpuLimit::SteamDeck => {
                                    Box::new(crate::settings::steam_deck::Cpus::from_json(
                                        settings.cpus.clone(),
                                        settings.version,
                                    ))
                                }
                                CpuLimit::SteamDeckAdvance => {
                                    Box::new(crate::settings::steam_deck::Cpus::from_json(
                                        settings.cpus.clone(),
                                        settings.version,
                                    ))
                                }
                                CpuLimit::Generic(x) => Box::new(crate::settings::generic::Cpus::<
                                    crate::settings::generic::Cpu,
                                >::from_json_and_limits(
                                    settings.cpus.clone(),
                                    settings.version,
                                    x,
                                )),
                                CpuLimit::GenericAMD(x) => Box::new(
                                    crate::settings::generic_amd::Cpus::from_json_and_limits(
                                        settings.cpus.clone(),
                                        settings.version,
                                        x,
                                    ),
                                ),
                                CpuLimit::Unknown => {
                                    Box::new(crate::settings::unknown::Cpus::from_json(
                                        settings.cpus.clone(),
                                        settings.version,
                                    ))
                                }
                            };
                            builder.cpus = Some(cpu_driver);
                        }
                        Limits::Gpu(gpu) => {
                            let driver: Box<dyn TGpu> = match gpu {
                                GpuLimit::SteamDeck => {
                                    Box::new(crate::settings::steam_deck::Gpu::from_json(
                                        settings.gpu.clone(),
                                        settings.version,
                                    ))
                                }
                                GpuLimit::SteamDeckAdvance => {
                                    Box::new(crate::settings::steam_deck::Gpu::from_json(
                                        settings.gpu.clone(),
                                        settings.version,
                                    ))
                                }
                                GpuLimit::Generic(x) => {
                                    Box::new(crate::settings::generic::Gpu::from_json_and_limits(
                                        settings.gpu.clone(),
                                        settings.version,
                                        x,
                                    ))
                                }
                                GpuLimit::GenericAMD(x) => Box::new(
                                    crate::settings::generic_amd::Gpu::from_json_and_limits(
                                        settings.gpu.clone(),
                                        settings.version,
                                        x,
                                    ),
                                ),
                                GpuLimit::Unknown => {
                                    Box::new(crate::settings::unknown::Gpu::from_json(
                                        settings.gpu.clone(),
                                        settings.version,
                                    ))
                                }
                            };
                            builder.gpu = Some(driver);
                        }
                        Limits::Battery(batt) => {
                            let driver: Box<dyn TBattery> = match batt {
                                BatteryLimit::SteamDeck => {
                                    Box::new(crate::settings::steam_deck::Battery::from_json(
                                        settings.battery.clone(),
                                        settings.version,
                                    ))
                                }
                                BatteryLimit::SteamDeckAdvance => {
                                    Box::new(crate::settings::steam_deck::Battery::from_json(
                                        settings.battery.clone(),
                                        settings.version,
                                    ))
                                }
                                BatteryLimit::Generic(x) => Box::new(
                                    crate::settings::generic::Battery::from_json_and_limits(
                                        settings.battery.clone(),
                                        settings.version,
                                        x,
                                    ),
                                ),
                                BatteryLimit::Unknown => {
                                    Box::new(crate::settings::unknown::Battery)
                                }
                            };
                            builder.battery = Some(driver);
                        }
                    }
                }
            } else {
                for limit in conf.limits {
                    match limit {
                        Limits::Cpu(cpus) => {
                            let cpu_driver: Box<dyn TCpus> = match cpus {
                                CpuLimit::SteamDeck => {
                                    Box::new(crate::settings::steam_deck::Cpus::system_default())
                                }
                                CpuLimit::SteamDeckAdvance => {
                                    Box::new(crate::settings::steam_deck::Cpus::system_default())
                                }
                                CpuLimit::Generic(x) => {
                                    Box::new(crate::settings::generic::Cpus::<
                                        crate::settings::generic::Cpu,
                                    >::from_limits(x))
                                }
                                CpuLimit::GenericAMD(x) => {
                                    Box::new(crate::settings::generic_amd::Cpus::from_limits(x))
                                }
                                CpuLimit::Unknown => {
                                    Box::new(crate::settings::unknown::Cpus::system_default())
                                }
                            };
                            builder.cpus = Some(cpu_driver);
                        }
                        Limits::Gpu(gpu) => {
                            let driver: Box<dyn TGpu> = match gpu {
                                GpuLimit::SteamDeck => {
                                    Box::new(crate::settings::steam_deck::Gpu::system_default())
                                }
                                GpuLimit::SteamDeckAdvance => {
                                    Box::new(crate::settings::steam_deck::Gpu::system_default())
                                }
                                GpuLimit::Generic(x) => {
                                    Box::new(crate::settings::generic::Gpu::from_limits(x))
                                }
                                GpuLimit::GenericAMD(x) => {
                                    Box::new(crate::settings::generic_amd::Gpu::from_limits(x))
                                }
                                GpuLimit::Unknown => {
                                    Box::new(crate::settings::unknown::Gpu::system_default())
                                }
                            };
                            builder.gpu = Some(driver);
                        }
                        Limits::Battery(batt) => {
                            let driver: Box<dyn TBattery> = match batt {
                                BatteryLimit::SteamDeck => {
                                    Box::new(crate::settings::steam_deck::Battery::system_default())
                                }
                                BatteryLimit::SteamDeckAdvance => {
                                    Box::new(crate::settings::steam_deck::Battery::system_default())
                                }
                                BatteryLimit::Generic(x) => {
                                    Box::new(crate::settings::generic::Battery::from_limits(x))
                                }
                                BatteryLimit::Unknown => {
                                    Box::new(crate::settings::unknown::Battery)
                                }
                            };
                            builder.battery = Some(driver);
                        }
                    }
                }
            }
        }
    }

    builder.build()
}

struct DriverBuilder {
    general: Box<dyn TGeneral>,
    cpus: Option<Box<dyn TCpus>>,
    gpu: Option<Box<dyn TGpu>>,
    battery: Option<Box<dyn TBattery>>,
}

impl DriverBuilder {
    fn new(json_path: std::path::PathBuf, profile_name: String) -> Self {
        Self {
            general: Box::new(General {
                persistent: false,
                path: json_path,
                name: profile_name,
                driver: DriverJson::AutoDetect,
                events: Default::default(),
            }),
            cpus: None,
            gpu: None,
            battery: None,
        }
    }

    fn is_complete(&self) -> bool {
        self.cpus.is_some() && self.gpu.is_some() && self.battery.is_some()
    }

    fn build(self) -> Driver {
        Driver {
            general: self.general,
            cpus: self
                .cpus
                .unwrap_or_else(|| Box::new(crate::settings::unknown::Cpus::system_default())),
            gpu: self
                .gpu
                .unwrap_or_else(|| Box::new(crate::settings::unknown::Gpu::system_default())),
            battery: self
                .battery
                .unwrap_or_else(|| Box::new(crate::settings::unknown::Battery)),
        }
    }
}
