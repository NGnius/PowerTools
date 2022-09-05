use crate::settings::SettingVariant;

pub struct StateError {
    pub msg: String,
    pub setting: Option<SettingVariant>,
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(setting) = self.setting {
            write!(f, "{} setting state error: {}", setting, self.msg)
        } else {
            write!(f, "State error: {}", self.msg)
        }
    }
}
