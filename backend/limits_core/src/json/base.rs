use std::default::Default;
use serde::{Deserialize, Serialize};

/// Base JSON limits information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Base {
    /// System-specific configurations
    pub configs: Vec<super::Config>,
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
            refresh: Some("http://limits.ngni.us:45000/powertools/v1".to_owned())
        }
    }
}
