use limits_core::json::{DeveloperMessage, Base};

pub fn limits_path() -> std::path::PathBuf {
    crate::utility::settings_dir().join(crate::consts::LIMITS_FILE)
}

// NOTE: eats errors
pub fn get_dev_messages() -> Vec<DeveloperMessage> {
    let limits_path = limits_path();
    if let Ok(file) = std::fs::File::open(&limits_path) {
        if let Ok(base) = serde_json::from_reader::<_, Base>(file) {
            base.messages
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}
