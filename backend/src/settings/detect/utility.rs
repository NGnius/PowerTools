pub fn limits_path() -> std::path::PathBuf {
    crate::utility::settings_dir().join(crate::consts::LIMITS_FILE)
}
