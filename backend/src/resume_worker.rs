use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::settings::{OnResume, Settings};
use crate::utility::unwrap_maybe_fatal;

const ALLOWED_ERROR: f64 = 100.0; // period of 10ms with 100x means sleep has to be >= 1s to be detected

pub fn spawn(settings: Settings) -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("resume_worker starting...");
        let duration = Duration::from_millis(10); // very low so it detects before Steam client does
        // this allows PowerTools to set some values at wakeup and Steam to override them before user notices
        let mut start = Instant::now();
        loop {
            let old_start = start.elapsed();
            start = Instant::now();
            if old_start.as_secs_f64() > duration.as_secs_f64() * (1.0 + ALLOWED_ERROR) {
                // has just resumed from sleep
                log::info!("Resume detected");
                unwrap_maybe_fatal(settings.on_resume(), "On resume failure");
                log::debug!(
                    "OnResume completed after sleeping for {}s",
                    old_start.as_secs_f32()
                );
            } else {
                log::debug!("OnResume got sleep period of {}s", old_start.as_secs_f32());
            }
            thread::sleep(duration);
        }
        //log::warn!("resume_worker completed!");
    })
}
