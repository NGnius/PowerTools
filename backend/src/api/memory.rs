use std::sync::{mpsc::Sender, Arc, Mutex};
use usdpl_back::core::serdes::Primitive;

use crate::settings::{Memory, OnSet};
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

/// Generate get THP enabled web method
pub fn get_transparent_hugepages_enabled(
    settings: Arc<Mutex<Memory>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "memory");
        vec![settings_lock
            .transparent_hugepages
            .or_else(|| Memory::read_transparent_hugepages_enabled().ok())
            .unwrap_or_default()
            .to_string()
            .into()]
    }
}

/// Generate set THP enabled web method
pub fn set_transparent_hugepages_enabled(
    settings: Arc<Mutex<Memory>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::String(new_val)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "memory");
            settings_lock.transparent_hugepages = new_val.parse().ok();
            unwrap_maybe_fatal(
                unwrap_lock(saver.lock(), "save channel").send(()),
                "Failed to send on save channel",
            );
            super::utility::map_empty_result(
                settings_lock.on_set(),
                settings_lock.transparent_hugepages.unwrap().to_string(),
            )
        } else {
            vec!["set_transparent_hugepages_enabled missing parameter".into()]
        }
    }
}

/// Generate unset THP enabled web method
pub fn unset_transparent_hugepages_enabled(
    settings: Arc<Mutex<Memory>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |_: super::ApiParameterType| {
        let mut settings_lock = unwrap_lock(settings.lock(), "memory");
        settings_lock.transparent_hugepages = None;
        unwrap_maybe_fatal(
            unwrap_lock(saver.lock(), "save channel").send(()),
            "Failed to send on save channel",
        );
        super::utility::map_empty_result(
            settings_lock.on_set(),
            Memory::read_transparent_hugepages_enabled()
                .ok()
                .unwrap_or_default()
                .to_string(),
        )
    }
}
