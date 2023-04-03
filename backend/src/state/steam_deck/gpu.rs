#[derive(Debug, Clone)]
pub struct Gpu {
    pub clock_limits_set: bool,
    pub fast_ppt_set: bool,
    pub slow_ppt_set: bool,
    pub is_resuming: bool,
}

impl std::default::Default for Gpu {
    fn default() -> Self {
        Self {
            clock_limits_set: true,
            fast_ppt_set: false,
            slow_ppt_set: false,
            is_resuming: false,
        }
    }
}
