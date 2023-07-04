use std::default::Default;
use serde::{Deserialize, Serialize};

/// Base JSON limits information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Base {
    /// System-specific configurations
    pub configs: Vec<super::Config>,
    /// Server messages
    pub messages: Vec<super::DeveloperMessage>,
    /// URL from which to grab the next update
    pub refresh: Option<String>,
}

impl Default for Base {
    fn default() -> Self {
        Base {
            configs: vec![
                super::Config {
                    name: "Steam Deck Custom".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t: AMD Custom APU 0405\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: Some("./pt_oc.json".into()),
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::SteamDeckAdvance),
                        super::Limits::Gpu(super::GpuLimit::SteamDeckAdvance),
                        super::Limits::Battery(super::BatteryLimit::SteamDeckAdvance),
                    ]
                },
                super::Config {
                    name: "Steam Deck".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t: AMD Custom APU 0405\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::SteamDeck),
                        super::Limits::Gpu(super::GpuLimit::SteamDeck),
                        super::Limits::Battery(super::BatteryLimit::SteamDeck),
                    ]
                },
                super::Config {
                    name: "AMD R3 2300U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 3 2300U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::GenericAMD(super::GenericCpuLimit {
                            clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(3700) }),
                            clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(3700) }),
                            clock_step: 100,
                        })),
                        super::Limits::Gpu(super::GpuLimit::GenericAMD(super::GenericGpuLimit {
                            fast_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(25_000_000) }),
                            slow_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(25_000_000) }),
                            ppt_step: Some(1_000_000),
                            clock_min: Some(super::RangeLimit { min: Some(400), max: Some(1100) }),
                            clock_max: Some(super::RangeLimit { min: Some(400), max: Some(1100) }),
                            clock_step: Some(100),
                            ..Default::default()
                        })),
                        super::Limits::Battery(super::BatteryLimit::Generic(super::GenericBatteryLimit{})),
                    ]
                },
                super::Config {
                    name: "AMD R5 5560U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 5 5560U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::GenericAMD(super::GenericCpuLimit {
                            clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(4000) }),
                            clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(4000) }),
                            clock_step: 100,
                        })),
                        super::Limits::Gpu(super::GpuLimit::GenericAMD(super::GenericGpuLimit {
                            fast_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(25_000_000) }),
                            slow_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(25_000_000) }),
                            ppt_step: Some(1_000_000),
                            clock_min: Some(super::RangeLimit { min: Some(400), max: Some(1600) }),
                            clock_max: Some(super::RangeLimit { min: Some(400), max: Some(1600) }),
                            clock_step: Some(100),
                            ..Default::default()
                        })),
                        super::Limits::Battery(super::BatteryLimit::Generic(super::GenericBatteryLimit{})),
                    ]
                },
                super::Config {
                    name: "AMD R7 5825U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 7 5825U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::GenericAMD(super::GenericCpuLimit {
                            clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(4500) }),
                            clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(4500) }),
                            clock_step: 100,
                        })),
                        super::Limits::Gpu(super::GpuLimit::GenericAMD(super::GenericGpuLimit {
                            fast_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(25_000_000) }),
                            slow_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(25_000_000) }),
                            ppt_step: Some(1_000_000),
                            clock_min: Some(super::RangeLimit { min: Some(400), max: Some(2000) }),
                            clock_max: Some(super::RangeLimit { min: Some(400), max: Some(2000) }),
                            clock_step: Some(100),
                            ..Default::default()
                        })),
                        super::Limits::Battery(super::BatteryLimit::Generic(super::GenericBatteryLimit{})),
                    ]
                },
                super::Config {
                    name: "AMD R7 6800U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 7 6800U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::Generic(super::GenericCpuLimit {
                            clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(4700) }),
                            clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(4700) }),
                            clock_step: 100,
                        })),
                        super::Limits::Gpu(super::GpuLimit::Generic(super::GenericGpuLimit {
                            fast_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(28_000_000) }),
                            slow_ppt: Some(super::RangeLimit { min: Some(1_000_000), max: Some(28_000_000) }),
                            ppt_step: Some(1_000_000),
                            clock_min: Some(super::RangeLimit { min: Some(400), max: Some(2200) }),
                            clock_max: Some(super::RangeLimit { min: Some(400), max: Some(2200) }),
                            clock_step: Some(100),
                            ..Default::default()
                        })),
                        super::Limits::Battery(super::BatteryLimit::Generic(super::GenericBatteryLimit{})),
                    ]
                },
                super::Config {
                    name: "Fallback".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: None,
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: vec![
                        super::Limits::Cpu(super::CpuLimit::Unknown),
                        super::Limits::Gpu(super::GpuLimit::Unknown),
                        super::Limits::Battery(super::BatteryLimit::Unknown),
                    ]
                }
            ],
            messages: vec![
                super::DeveloperMessage {
                    id: 1,
                    title: "Welcome".to_owned(),
                    body: "Thanks for installing PowerTools! For more information, please check the wiki. For bugs and requests, please create an issue on GitHub.".to_owned(),
                    url: Some("https://github.com/NGnius/PowerTools/wiki".to_owned()),
                }
            ],
            refresh: Some("http://limits.ngni.us:45000/powertools/v1".to_owned())
        }
    }
}
