use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RangeLimit<T> {
    pub min: T,
    pub max: T,
}

#[derive(Serialize, Deserialize)]
pub struct SettingsLimits {
    pub battery: BatteryLimits,
    pub cpu: CpusLimits,
    pub gpu: GpuLimits,
    pub general: GeneralLimits,
}

#[derive(Serialize, Deserialize)]
pub struct BatteryLimits {
    pub charge_current: Option<RangeLimit<u64>>,
    pub charge_current_step: u64,
    pub charge_modes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CpusLimits {
    pub cpus: Vec<CpuLimits>,
    pub count: usize,
    pub smt_capable: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CpuLimits {
    pub clock_min_limits: Option<RangeLimit<u64>>,
    pub clock_max_limits: Option<RangeLimit<u64>>,
    pub clock_step: u64,
    pub governors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralLimits {
}

#[derive(Serialize, Deserialize)]
pub struct GpuLimits {
    pub fast_ppt_limits: Option<RangeLimit<u64>>,
    pub slow_ppt_limits: Option<RangeLimit<u64>>,
    pub ppt_step: u64,
    pub tdp_limits: Option<RangeLimit<u64>>,
    pub tdp_boost_limits: Option<RangeLimit<u64>>,
    pub tdp_step: u64,
    pub clock_min_limits: Option<RangeLimit<u64>>,
    pub clock_max_limits: Option<RangeLimit<u64>>,
    pub clock_step: u64,
    pub memory_control_capable: bool,
}
