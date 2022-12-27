use std::default::Default;

use serde::{Deserialize, Serialize};

use super::JsonError;
use super::{BatteryJson, CpuJson, GpuJson, MemoryJson};

#[derive(Serialize, Deserialize)]
pub struct SettingsJson {
    pub version: u64,
    pub name: String,
    pub persistent: bool,
    pub cpus: Vec<CpuJson>,
    pub gpu: GpuJson,
    pub battery: BatteryJson,
    pub memory: MemoryJson,
}

impl Default for SettingsJson {
    fn default() -> Self {
        Self {
            version: 0,
            name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
            persistent: false,
            cpus: Vec::with_capacity(8),
            gpu: GpuJson::default(),
            battery: BatteryJson::default(),
            memory: MemoryJson::default(),
        }
    }
}

impl SettingsJson {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), JsonError> {
        let path = path.as_ref();

        if self.persistent {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(JsonError::Io)?;
            }
            let mut file = std::fs::File::create(path).map_err(JsonError::Io)?;
            serde_json::to_writer_pretty(&mut file, &self).map_err(JsonError::Serde)
        } else {
            if path.exists() {
                // remove settings file when persistence is turned off, to prevent it from be loaded next time.
                std::fs::remove_file(path).map_err(JsonError::Io)
            } else {
                Ok(())
            }
        }
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, JsonError> {
        let mut file = std::fs::File::open(path).map_err(JsonError::Io)?;
        serde_json::from_reader(&mut file).map_err(JsonError::Serde)
    }
}

#[derive(Serialize, Deserialize)]
pub struct MinMaxJson<T> {
    pub max: T,
    pub min: T,
}
