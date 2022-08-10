use super::SettingError;

pub trait OnSet {
    fn on_set(&mut self) -> Result<(), SettingError>;
}

pub trait OnResume {
    fn on_resume(&self) -> Result<(), SettingError>;
}

pub trait SettingsRange {
    fn max() -> Self;
    fn min() -> Self;
}
