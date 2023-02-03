use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "limits")]
pub enum Limits {
    Cpu(super::CpuLimit),
    Gpu(super::GpuLimit),
    Battery(super::BatteryLimit),
}
