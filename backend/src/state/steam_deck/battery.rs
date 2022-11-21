#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate_set: bool,
    pub charge_mode_set: bool,
}

impl std::default::Default for Battery {
    fn default() -> Self {
        Self {
            charge_rate_set: false,
            charge_mode_set: false,
        }
    }
}
