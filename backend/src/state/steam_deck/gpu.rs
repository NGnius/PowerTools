#[derive(Debug, Clone)]
pub struct Gpu {
    pub clock_limits_set: bool,
    pub is_resuming: bool,
}

impl std::default::Default for Gpu {
    fn default() -> Self {
        Self {
            clock_limits_set: false,
            is_resuming: false,
        }
    }
}
