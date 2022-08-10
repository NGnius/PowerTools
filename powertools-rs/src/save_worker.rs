use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::persist::SettingsJson;
use crate::settings::Settings;
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

pub fn spawn(settings: Settings) -> (JoinHandle<()>, Sender<()>) {
    let (sender, receiver): (Sender<()>, Receiver<()>) = mpsc::channel();
    let worker = thread::spawn(move || {
        for _ in receiver.iter() {
            let save_path = unwrap_lock(settings.general.lock(), "general").path.clone();
            let save_json: SettingsJson = settings.clone().into();
            unwrap_maybe_fatal(save_json.save(&save_path), "Failed to save settings");
            log::debug!("Saved settings to {}", save_path.display());
        }
    });
    (worker, sender)
}
