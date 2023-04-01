#[derive(Debug, Clone)]
pub struct Cpu {
    pub clock_limits_set: bool,
    pub is_resuming: bool,
    pub do_set_online: bool,
}

impl std::default::Default for Cpu {
    fn default() -> Self {
        Self {
            clock_limits_set: true,
            is_resuming: false,
            do_set_online: true,
        }
    }
}
