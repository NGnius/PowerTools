use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::api::handler::ApiMessage;
//use crate::utility::unwrap_maybe_fatal;

const PERIOD: Duration = Duration::from_secs(5);

pub fn spawn(sender: Sender<ApiMessage>) -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("power_worker starting...");
        loop {
            sender
                .send(ApiMessage::PowerVibeCheck)
                .expect("power_worker send failed");
            thread::sleep(PERIOD);
        }
        //log::warn!("resume_worker completed!");
    })
}
