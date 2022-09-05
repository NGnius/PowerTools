use super::SettingVariant;

#[derive(Debug, Clone)]
pub struct SettingError {
    pub msg: String,
    pub setting: SettingVariant,
}

impl std::fmt::Display for SettingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} setting error: {}", self.setting, self.msg)
    }
}
