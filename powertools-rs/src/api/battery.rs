use std::sync::{mpsc::Sender, Arc, Mutex};
use usdpl_back::core::serdes::Primitive;

use crate::settings::{Battery, OnSet};
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

/// Current current (ha!) web method
pub fn current_now(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(crate::settings::Battery::current_now())
}

/// Generate set battery charge rate web method
pub fn set_charge_rate(
    settings: Arc<Mutex<Battery>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(new_val)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "battery");
            settings_lock.charge_rate = Some(*new_val as _);
            unwrap_maybe_fatal(
                unwrap_lock(saver.lock(), "save channel").send(()),
                "Failed to send on save channel",
            );
            super::utility::map_empty_result(
                settings_lock.on_set(),
                settings_lock.charge_rate.unwrap(),
            )
        } else {
            vec!["set_charge_rate missing parameter".into()]
        }
    }
}

/// Generate get battery charge rate web method
pub fn get_charge_rate(
    settings: Arc<Mutex<Battery>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "battery");
        vec![settings_lock
            .charge_rate
            .map(|x| x.into())
            .unwrap_or(Primitive::Empty)]
    }
}

/// Generate unset battery charge rate web method
pub fn unset_charge_rate(
    settings: Arc<Mutex<Battery>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |_: super::ApiParameterType| {
        let mut settings_lock = unwrap_lock(settings.lock(), "battery");
        settings_lock.charge_rate = None;
        unwrap_maybe_fatal(
            unwrap_lock(saver.lock(), "save channel").send(()),
            "Failed to send on save channel",
        );
        vec![]
    }
}
