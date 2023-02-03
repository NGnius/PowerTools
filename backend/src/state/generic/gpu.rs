#[derive(Debug, Clone)]
pub struct Gpu {
    pub clock_limits_set: bool,
    pub old_fast_ppt: Option<u64>,
    pub old_slow_ppt: Option<u64>,
}

impl std::default::Default for Gpu {
    fn default() -> Self {
        Self {
            clock_limits_set: false,
            old_fast_ppt: None,
            old_slow_ppt: None,
        }
    }
}
