use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::persist::SettingsJson;
use crate::settings::Settings;
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

pub fn spawn(settings: Settings) -> (JoinHandle<()>, Sender<()>) {
    let (sender, receiver): (Sender<()>, Receiver<()>) = mpsc::channel();
    let worker = thread::spawn(move || {
        log::info!("save_worker starting...");
        for _ in receiver.iter() {
            log::debug!("save_worker is saving...");
            let is_persistent = unwrap_lock(settings.general.lock(), "general").persistent.clone();
            if is_persistent {
                let save_path = crate::utility::settings_dir()
                    .join(unwrap_lock(settings.general.lock(), "general").path.clone());
                let settings_clone = settings.clone();
                let save_json: SettingsJson = settings_clone.into();
                unwrap_maybe_fatal(save_json.save(&save_path), "Failed to save settings");
                log::debug!("Saved settings to {}", save_path.display());
            } else {
                log::debug!("Ignored save request for non-persistent settings");
            }
        }
        log::warn!("save_worker completed!");
    });
    (worker, sender)
}
