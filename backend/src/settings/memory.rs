use std::convert::Into;
use std::fmt::{Display};
use std::str::FromStr;

use super::{OnResume, OnSet, SettingError};
use crate::persist::MemoryJson;

#[derive(Debug, Clone)]
pub struct Memory {
    pub transparent_hugepages: Option<TransparentHugepages>,
    state: crate::state::Memory,
}

const MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH: &str = "/sys/kernel/mm/transparent_hugepage/enabled";

impl Memory {
    #[inline]
    pub fn from_json(other: MemoryJson, version: u64) -> Self {
        match version {
            0 => Self {
                transparent_hugepages: other.transparent_hugepages.and_then(|v| TransparentHugepages::from_str(&v).ok()),
                state: crate::state::Memory::default(),
            },
            _ => Self {
                transparent_hugepages: other.transparent_hugepages.and_then(|v| TransparentHugepages::from_str(&v).ok()),
                state: crate::state::Memory::default(),
            },
        }
    }

    fn set_all(&mut self) -> Result<(), SettingError> {
        if let Some(thp) = self.transparent_hugepages {
            self.state.transparent_hugepages_set = true;
            usdpl_back::api::files::write_single(MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH, thp.to_string()).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH, e),
                    setting: super::SettingVariant::Memory,
                },
            )
        } else if self.state.transparent_hugepages_set {
            self.state.transparent_hugepages_set = false;
            usdpl_back::api::files::write_single(MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH, TransparentHugepages::default().to_string()).map_err(
                |e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH, e),
                    setting: super::SettingVariant::Memory,
                },
            )
        } else {
            Ok(())
        }
    }

    pub fn system_default() -> Self {
        Self {
            transparent_hugepages: Some(usdpl_back::api::files::read_single(MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH).unwrap_or_default()),
            state: crate::state::Memory::default(),
        }
    }

    pub fn read_transparent_hugepages_enabled() -> Result<TransparentHugepages, SettingError> {
        match usdpl_back::api::files::read_single::<_, TransparentHugepages, _>(MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH) {
            Ok(val) => Ok(val),
            Err((Some(e), None)) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH, e),
                setting: super::SettingVariant::Battery,
            }),
            Err((None, Some(e))) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH, e),
                setting: super::SettingVariant::Battery,
            }),
            Err(_) => panic!(
                "Invalid error while reading from `{}`",
                MM_TRANSPARENT_HUGEPAGES_ENABLED_PATH
            ),
        }
    }
}

impl Into<MemoryJson> for Memory {
    #[inline]
    fn into(self) -> MemoryJson {
        MemoryJson {
            transparent_hugepages: self.transparent_hugepages.map(|v| v.to_string()),
        }
    }
}

impl OnSet for Memory {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.set_all()
    }
}

impl OnResume for Memory {
    fn on_resume(&self) -> Result<(), SettingError> {
        let mut copy = self.clone();
        copy.set_all()
    }
}

/// Error for FromStr impl of TransparentHugepages.
pub enum TransparentHugepagesParseError {
    UnknownOption(String),
}

impl Display for TransparentHugepagesParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOption(s) => write!(f, "unknown transparent hugepages option: {}", s),
        }
    }
}

/// Options for transparent hugepages.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TransparentHugepages {
    Always,
    MAdvise,
    Never, // Do not use - https://serverfault.com/a/896131
}

impl Default for TransparentHugepages {
    fn default() -> Self {
        TransparentHugepages::MAdvise
    }
}

impl FromStr for TransparentHugepages {
    type Err = TransparentHugepagesParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // If the format is from the sysfs file, try to extract it out.
        // Example: always [madvise] never
        let has_brackets = s.contains("[") && s.contains("]");
        if has_brackets {
            let bracketed: String = s.chars()
                .skip_while(|c| *c != '[')
                .skip(1)
                .take_while(|c| *c != ']')
                .collect();

            return TransparentHugepages::from_str(&bracketed)
        }

        // Selected an enum variant from the string's value.
        use TransparentHugepages::*;
        match s {
            "always" => Ok(Always),
            "madvise" => Ok(MAdvise),
            "never" => Ok(Never),
            _ => Err(TransparentHugepagesParseError::UnknownOption(s.to_owned())),
        }
    }
}

impl ToString for TransparentHugepages {
    fn to_string(&self) -> String {
        use TransparentHugepages::*;
        match self {
            Always => "always",
            MAdvise => "madvise",
            Never => "never",
        }.to_owned()
    }
}
