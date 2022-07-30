use std::convert::{Into, From};

use crate::persist::SettingsJson;
use super::{Cpu, Gpu};

const LATEST_VERSION: u64 = 0;

#[derive(Debug, Clone)]
pub struct Settings {
    pub persistent: bool,
    pub cpus: Vec<Cpu>,
    pub gpu: Gpu,
}

impl From<SettingsJson> for Settings {
    #[inline]
    fn from(mut other: SettingsJson) -> Self {
        match other.version {
            0 => Self {
                persistent: other.persistent,
                cpus: other.cpus.drain(..).map(|cpu| Cpu::from_json(cpu, other.version)).collect(),
                gpu: Gpu::from_json(other.gpu, other.version),
            },
            _ => Self {
                persistent: other.persistent,
                cpus: other.cpus.drain(..).map(|cpu| Cpu::from_json(cpu, other.version)).collect(),
                gpu: Gpu::from_json(other.gpu, other.version),
            }
        }
    }
}

impl Into<SettingsJson> for Settings {
    #[inline]
    fn into(mut self) -> SettingsJson {
        SettingsJson {
            version: LATEST_VERSION,
            persistent: self.persistent,
            cpus: self.cpus.drain(..).map(|cpu| cpu.into()).collect(),
            gpu: self.gpu.into()
        }
    }
}
