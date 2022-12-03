use std::thread::{self, JoinHandle};
#[cfg(feature = "online")]
use std::time::Duration;

use limits_core::json::Base;

#[cfg(feature = "online")]
pub fn spawn() -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("limits_worker starting...");
        let sleep_dur = Duration::from_secs(60*60*24); // 1 day
        let limits_path = super::utility::limits_path();
        loop {
            thread::sleep(sleep_dur);
            if (limits_path.exists() && limits_path.is_file()) || !limits_path.exists() {
                // try to load limits from file, fallback to built-in default
                let base = match std::fs::File::open(&limits_path) {
                    Ok(f) => {
                        match serde_json::from_reader(f) {
                            Ok(b) => b,
                            Err(e) => {
                                log::error!("Cannot parse {}: {}", limits_path.display(), e);
                                Base::default()
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Cannot open {}: {}", limits_path.display(), e);
                        Base::default()
                    }
                };
                if let Some(refresh) = &base.refresh {
                    // try to retrieve newer version
                    match ureq::get(refresh)
                        .call() {
                        Ok(response) => {
                            let json_res: std::io::Result<Base> = response.into_json();
                            match json_res {
                                Ok(new_base) => {
                                    match std::fs::File::create(&limits_path) {
                                        Ok(f) => {
                                            match serde_json::to_writer_pretty(f, &new_base) {
                                                Ok(_) => log::info!("Successfully updated limits from `{}`, cached at {}", refresh, limits_path.display()),
                                                Err(e) => log::error!("Failed to save limits json to file `{}`: {}", limits_path.display(), e),
                                            }
                                        },
                                        Err(e) => log::error!("Cannot create {}: {}", limits_path.display(), e)
                                    }
                                },
                                Err(e) => log::error!("Cannot parse response from `{}`: {}", refresh, e),
                            }
                        },
                        Err(e) => log::warn!("Cannot download limits from `{}`: {}", refresh, e),
                    }
                } else {
                    log::info!("limits_worker refresh is empty, terminating...");
                    break;
                }
            } else if !limits_path.is_file() {
                log::error!("Path for storing limits is not a file!");
            }
        }
        log::warn!("limits_worker completed!");
    })
}

#[cfg(not(feature = "online"))]
pub fn spawn() -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("limits_worker disabled...");
    })
}

pub fn get_limits_blocking() -> Base {
    let limits_path = super::utility::limits_path();
    if limits_path.is_file() {
        match std::fs::File::open(&limits_path) {
            Ok(f) => {
                match serde_json::from_reader(f) {
                    Ok(b) => b,
                    Err(e) => {
                        log::error!("Cannot parse {}: {}", limits_path.display(), e);
                        Base::default()
                    }
                }
            },
            Err(e) => {
                log::error!("Cannot open {}: {}", limits_path.display(), e);
                Base::default()
            }
        }
    } else {
        #[cfg(feature = "online")]
        {
            let refresh = Base::default().refresh.unwrap();
            match ureq::get(&refresh) // try to retrieve newer version
                .call() {
                Ok(response) => {
                    let json_res: std::io::Result<Base> = response.into_json();
                    match json_res {
                        Ok(new_base) => return new_base,
                        Err(e) => log::error!("Cannot parse response from `{}`: {}", refresh, e)
                    }
                },
                Err(e) => log::warn!("Cannot download limits from `{}`: {}", refresh, e),
            }
        }
        Base::default()
    }
}
