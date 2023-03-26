#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate_set: bool,
    pub charge_mode_set: bool,
    pub charger_state: ChargeState,
}

impl std::default::Default for Battery {
    fn default() -> Self {
        Self {
            charge_rate_set: true,
            charge_mode_set: true,
            charger_state: ChargeState::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChargeState {
    PluggedIn,
    Unplugged,
    Unknown,
}
