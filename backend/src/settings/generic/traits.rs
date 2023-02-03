use limits_core::json::GenericCpuLimit;
use crate::persist::CpuJson;

pub trait FromGenericCpuInfo {
    fn from_limits(cpu_index: usize, limits: GenericCpuLimit) -> Self;

    fn from_json_and_limits(other: CpuJson, version: u64, cpu_index: usize, limits: GenericCpuLimit) -> Self;
}
