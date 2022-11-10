use std::fmt::Display;
//use std::sync::{LockResult, MutexGuard};

pub fn unwrap_maybe_fatal<T: Sized, E: Display>(result: Result<T, E>, message: &str) -> T {
    match result {
        Ok(x) => x,
        Err(e) => {
            log::error!("{}: {}", message, e);
            panic!("{}: {}", message, e);
        }
    }
}

/*pub fn unwrap_lock<'a, T: Sized>(
    result: LockResult<MutexGuard<'a, T>>,
    lock_name: &str,
) -> MutexGuard<'a, T> {
    match result {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed to acquire {} lock: {}", lock_name, e);
            panic!("Failed to acquire {} lock: {}", lock_name, e);
        }
    }
}*/

pub fn settings_dir() -> std::path::PathBuf {
    usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/home/deck".into())
        .join(".config/powertools/")
}
