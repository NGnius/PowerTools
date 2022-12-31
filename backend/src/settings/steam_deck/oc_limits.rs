use serde::{Deserialize, Serialize};
use crate::settings::MinMax;

const OC_LIMITS_FILEPATH: &str = "./pt_oc.json";

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct OverclockLimits {
    pub battery: BatteryLimits,
    pub cpus: CpusLimits,
    pub gpu: GpuLimits,
}

impl Default for OverclockLimits {
    fn default() -> Self {
        Self {
            battery: BatteryLimits::default(),
            cpus: CpusLimits::default(),
            gpu: GpuLimits::default(),
        }
    }
}

impl OverclockLimits {
    pub fn load_or_default() -> Self {
        let path = std::path::Path::new(OC_LIMITS_FILEPATH);
        if path.exists() {
            log::info!("Steam Deck limits file {} found", path.display());
            let mut file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("Steam Deck limits file {} err: {} (using default fallback)", path.display(), e);
                    return Self::default();
                },
            };
            match serde_json::from_reader(&mut file) {
                Ok(result) => {
                    log::debug!("Steam Deck limits file {} successfully loaded", path.display());
                    result
                },
                Err(e) => {
                    log::warn!("Steam Deck limits file {} json err: {} (using default fallback)", path.display(), e);
                    Self::default()
                }
            }
        } else {
            log::info!("Steam Deck limits file {} not found (using default fallback)", path.display());
            Self::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct BatteryLimits {
    pub charge_rate: MinMax<u64>,
}

impl Default for BatteryLimits {
    fn default() -> Self {
        Self {
            charge_rate: MinMax { min: 250, max: 2500 },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct CpusLimits {
    pub cpus: Vec<CpuLimits>,
}

impl Default for CpusLimits {
    fn default() -> Self {
        Self {
            cpus: [(); 8].iter().map(|_| CpuLimits::default()).collect()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct CpuLimits {
    pub clock_min: MinMax<u64>,
    pub clock_max: MinMax<u64>,
}

impl Default for CpuLimits {
    fn default() -> Self {
        Self {
            clock_min: MinMax { min: 1400, max: 3500 },
            clock_max: MinMax { min: 500, max: 3500 }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct GpuLimits {
    pub fast_ppt: MinMax<u64>,
    pub slow_ppt: MinMax<u64>,
    pub clock_min: MinMax<u64>,
    pub clock_max: MinMax<u64>,
}

impl Default for GpuLimits {
    fn default() -> Self {
        Self {
            fast_ppt: MinMax { min: 1000000, max: 30_000_000 },
            slow_ppt: MinMax { min: 1000000, max: 29_000_000 },
            clock_min: MinMax { min: 200, max: 1600 },
            clock_max: MinMax { min: 200, max: 1600 }
        }
    }
}
