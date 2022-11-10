use std::thread::{self, JoinHandle};

use crate::settings::Settings;
//use crate::utility::{unwrap_lock, unwrap_maybe_fatal};
use crate::api::handler::ApiMessageHandler;

pub fn spawn(mut settings: Settings, mut handler: ApiMessageHandler) -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("api_worker starting...");
        handler.process_forever(&mut settings);
        log::warn!("api_worker completed!");
    })
}
