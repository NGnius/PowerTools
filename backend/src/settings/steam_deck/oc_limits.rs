use crate::api::RangeLimit as MinMax;
use serde::{Deserialize, Serialize};

const OC_LIMITS_FILEPATH: &str = "pt_oc.json";

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
    /// (Self, is_default)
    pub fn load_or_default() -> (Self, bool) {
        let path = oc_limits_filepath();
        if path.exists() {
            log::info!("Steam Deck limits file {} found", path.display());
            let mut file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    log::warn!(
                        "Steam Deck limits file {} err: {} (using default fallback)",
                        path.display(),
                        e
                    );
                    return (Self::default(), true);
                }
            };
            match serde_json::from_reader(&mut file) {
                Ok(result) => {
                    log::debug!(
                        "Steam Deck limits file {} successfully loaded",
                        path.display()
                    );
                    (result, false)
                }
                Err(e) => {
                    log::warn!(
                        "Steam Deck limits file {} json err: {} (using default fallback)",
                        path.display(),
                        e
                    );
                    (Self::default(), true)
                }
            }
        } else {
            log::info!(
                "Steam Deck limits file {} not found (using default fallback)",
                path.display()
            );
            (Self::default(), true)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct BatteryLimits {
    pub charge_rate: MinMax<u64>,
    pub extra_readouts: bool,
}

impl Default for BatteryLimits {
    fn default() -> Self {
        Self {
            charge_rate: MinMax {
                min: 250,
                max: 2500,
            },
            extra_readouts: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct CpusLimits {
    pub cpus: Vec<CpuLimits>,
    pub global_governors: bool,
}

impl Default for CpusLimits {
    fn default() -> Self {
        Self {
            cpus: [(); 8].iter().map(|_| CpuLimits::default()).collect(),
            global_governors: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct CpuLimits {
    pub clock_min: MinMax<u64>,
    pub clock_max: MinMax<u64>,
    pub clock_step: u64,
    pub skip_resume_reclock: bool,
}

impl Default for CpuLimits {
    fn default() -> Self {
        Self {
            clock_min: MinMax {
                min: 1400,
                max: 3500,
            },
            clock_max: MinMax {
                min: 400,
                max: 3500,
            },
            clock_step: 100,
            skip_resume_reclock: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct GpuLimits {
    pub fast_ppt: MinMax<u64>,
    pub fast_ppt_default: u64,
    pub slow_ppt: MinMax<u64>,
    pub slow_ppt_default: u64,
    pub ppt_divisor: u64,
    pub ppt_step: u64,
    pub clock_min: MinMax<u64>,
    pub clock_max: MinMax<u64>,
    pub clock_step: u64,
    pub skip_resume_reclock: bool,
}

impl Default for GpuLimits {
    fn default() -> Self {
        Self {
            fast_ppt: MinMax {
                min: 1000000,
                max: 30_000_000,
            },
            fast_ppt_default: 15_000_000,
            slow_ppt: MinMax {
                min: 1000000,
                max: 29_000_000,
            },
            slow_ppt_default: 15_000_000,
            ppt_divisor: 1_000_000,
            ppt_step: 1,
            clock_min: MinMax {
                min: 400,
                max: 1600,
            },
            clock_max: MinMax {
                min: 400,
                max: 1600,
            },
            clock_step: 100,
            skip_resume_reclock: false,
        }
    }
}

fn oc_limits_filepath() -> std::path::PathBuf {
    crate::utility::settings_dir().join(OC_LIMITS_FILEPATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "dev_stuff"))] // this can fail due to reading from incompletely-written file otherwise
    #[test]
    fn load_pt_oc() {
        let mut file = std::fs::File::open("../pt_oc.json").unwrap();
        let settings: OverclockLimits = serde_json::from_reader(&mut file).unwrap();
        assert!(settings.cpus.cpus.len() == 8);
    }

    #[cfg(feature = "dev_stuff")]
    #[test]
    fn emit_default_pt_oc() {
        let mut file = std::fs::File::create("../pt_oc.json").unwrap();
        serde_json::to_writer_pretty(&mut file, &OverclockLimits::default()).unwrap();
    }
}
