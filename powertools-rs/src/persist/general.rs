use std::default::Default;
use std::fmt::Display;

use serde::{Serialize, Deserialize};

use super::{CpuJson, GpuJson};

#[derive(Serialize, Deserialize)]
pub struct SettingsJson {
    pub version: u64,
    pub persistent: bool,
    pub cpus: Vec<CpuJson>,
    pub gpu: GpuJson,
}

impl Default for SettingsJson {
    fn default() -> Self {
        Self {
            version: 0,
            persistent: false,
            cpus: Vec::with_capacity(8),
            gpu: GpuJson::default(),
        }
    }
}

impl SettingsJson {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), JsonError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(JsonError::Io)?;
        }
        let mut file = std::fs::File::create(path).map_err(JsonError::Io)?;
        serde_json::to_writer_pretty(&mut file, &self).map_err(JsonError::Serde)
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, JsonError> {
        let mut file = std::fs::File::open(path).map_err(JsonError::Io)?;
        serde_json::from_reader(&mut file).map_err(JsonError::Serde)
    }
}

#[derive(Debug)]
pub enum JsonError {
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Serde(e) => (e as &dyn Display).fmt(f),
            Self::Io(e) => (e as &dyn Display).fmt(f),
        }
    }
}
