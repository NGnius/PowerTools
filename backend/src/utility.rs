use std::fmt::Display;
//use std::sync::{LockResult, MutexGuard};
//use std::fs::{Permissions, metadata};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;

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
        .unwrap_or_else(|| "/tmp/".into())
        .join(".config/powertools/")
}

pub fn chown_settings_dir() -> std::io::Result<()> {
    let dir = settings_dir();
    #[cfg(feature = "decky")]
    let deck_user = usdpl_back::api::decky::user().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Decky missing deck user's username",
        )
    })?;
    #[cfg(not(feature = "decky"))]
    let deck_user = "deck".to_owned();
    // FIXME this shouldn't need to invoke a command
    let output = std::process::Command::new("id")
        .args(["-u", &deck_user])
        .output()?;
    let uid: u32 = String::from_utf8_lossy(&output.stdout)
        .parse()
        .unwrap_or(1000);
    log::info!(
        "chmod/chown ~/.config/powertools for user `{}` ({})",
        deck_user,
        uid
    );
    let permissions = PermissionsExt::from_mode(0o755);
    std::fs::set_permissions(&dir, permissions)?;
    // FIXME once merged into stable https://github.com/rust-lang/rust/issues/88989
    //std::os::unix::fs::chown(&dir, Some(uid), Some(uid))
    std::process::Command::new("chown")
        .args([
            "-R",
            &format!("{}:{}", deck_user, deck_user),
            &dir.to_str().unwrap_or("."),
        ])
        .output()?;
    Ok(())
}

fn version_filepath() -> std::path::PathBuf {
    settings_dir().join(".version")
}

pub fn save_version_file() -> std::io::Result<usize> {
    let path = version_filepath();
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    std::fs::File::create(path)?.write(crate::consts::PACKAGE_VERSION.as_bytes())
}

pub fn read_version_file() -> String {
    let path = version_filepath();
    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut read_version = String::new();
            match file.read_to_string(&mut read_version) {
                Ok(_) => read_version,
                Err(e) => {
                    log::warn!("Cannot read version file str: {}", e);
                    crate::consts::PACKAGE_VERSION.to_owned()
                }
            }
        }
        Err(e) => {
            log::warn!("Cannot read version file: {}", e);
            crate::consts::PACKAGE_VERSION.to_owned()
        }
    }
}
