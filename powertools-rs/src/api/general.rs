use std::sync::{mpsc::Sender, Arc, Mutex};
use usdpl_back::core::serdes::Primitive;

use crate::settings::{General, Settings, OnSet};
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

/// Generate set persistent web method
pub fn set_persistent(
    settings: Arc<Mutex<General>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::Bool(new_val)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "general");
            settings_lock.persistent = *new_val;
            unwrap_maybe_fatal(
                unwrap_lock(saver.lock(), "save channel").send(()),
                "Failed to send on save channel",
            );
            super::utility::map_empty_result(
                settings_lock.on_set(),
                settings_lock.persistent,
            )
        } else {
            vec!["set_persistent missing parameter".into()]
        }
    }
}

/// Generate get persistent save mode web method
pub fn get_persistent(
    settings: Arc<Mutex<General>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "general");
        vec![settings_lock
            .persistent.into()]
    }
}

/// Generate load app settings from file web method
pub fn load_settings(
    settings: Settings,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::String(path)) = params_in.get(0) {
            if let Some(Primitive::String(name)) = params_in.get(1) {
                super::utility::map_result(
                    settings.load_file(path.into(), name.to_owned())
                )
            } else {
                vec!["load_settings missing name parameter".into()]
            }
            //let mut general_lock = unwrap_lock(settings.general.lock(), "general");
        } else {
            vec!["load_settings missing path parameter".into()]
        }
    }
}

/// Generate get current settings name
pub fn get_name(
    settings: Arc<Mutex<General>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "general");
        vec![settings_lock
            .name
            .clone()
            .into()]
    }
}
