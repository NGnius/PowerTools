use std::sync::{mpsc::Sender, Arc, Mutex};
use usdpl_back::core::serdes::Primitive;

use crate::settings::{Gpu, OnSet, MinMax};
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

pub fn set_ppt(
    settings: Arc<Mutex<Gpu>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(fast_ppt)) = params_in.get(0) {
            if let Some(Primitive::F64(slow_ppt)) = params_in.get(1) {
                let mut settings_lock = unwrap_lock(settings.lock(), "gpu");
                settings_lock.fast_ppt = Some(*fast_ppt as u64);
                settings_lock.slow_ppt = Some(*slow_ppt as u64);
                unwrap_maybe_fatal(
                    unwrap_lock(saver.lock(), "save channel").send(()),
                    "Failed to send on save channel",
                );
                match settings_lock.on_set() {
                    Ok(_) => vec![
                        settings_lock.fast_ppt.unwrap().into(),
                        settings_lock.slow_ppt.unwrap().into()
                    ],
                    Err(e) => vec![e.msg.into()],
                }
            } else {
                vec!["set_ppt missing parameter 1".into()]
            }
        } else {
            vec!["set_ppt missing parameter 0".into()]
        }
    }
}

pub fn get_ppt(
    settings: Arc<Mutex<Gpu>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "gpu");
        let fast_ppt = settings_lock.fast_ppt.map(|x| x.into()).unwrap_or(Primitive::Empty);
        let slow_ppt = settings_lock.slow_ppt.map(|x| x.into()).unwrap_or(Primitive::Empty);
        vec![fast_ppt, slow_ppt]
    }
}

pub fn unset_ppt(
    settings: Arc<Mutex<Gpu>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |_: super::ApiParameterType| {
        let mut settings_lock = unwrap_lock(settings.lock(), "gpu");
        settings_lock.fast_ppt = None;
        settings_lock.slow_ppt = None;
        unwrap_maybe_fatal(
            unwrap_lock(saver.lock(), "save channel").send(()),
            "Failed to send on save channel",
        );
        super::utility::map_empty_result(
            settings_lock.on_set(),
            Primitive::Empty,
        )
    }
}

pub fn set_clock_limits(
    settings: Arc<Mutex<Gpu>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(min)) = params_in.get(0) {
            if let Some(Primitive::F64(max)) = params_in.get(1) {
                let mut settings_lock = unwrap_lock(settings.lock(), "gpu");
                settings_lock.clock_limits = Some(MinMax {
                    min: *min as _,
                    max: *max as _,
                });
                unwrap_maybe_fatal(
                    unwrap_lock(saver.lock(), "save channel").send(()),
                    "Failed to send on save channel",
                );
                match settings_lock.on_set() {
                    Ok(_) => vec![
                        settings_lock.clock_limits.as_ref().unwrap().min.into(),
                        settings_lock.clock_limits.as_ref().unwrap().max.into(),
                    ],
                    Err(e) => vec![e.msg.into()]
                }
            } else {
                vec!["set_clock_limits missing parameter 1".into()]
            }
        } else {
            vec!["set_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn get_clock_limits(
    settings: Arc<Mutex<Gpu>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "gpu");
        if let Some(min_max) = &settings_lock.clock_limits {
            vec![min_max.max.into(), min_max.min.into()]
        } else {
            vec![Primitive::Empty, Primitive::Empty]
        }
    }
}

pub fn unset_clock_limits(
    settings: Arc<Mutex<Gpu>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |_: super::ApiParameterType| {
        let mut settings_lock = unwrap_lock(settings.lock(), "gpu");
        settings_lock.clock_limits = None;
        unwrap_maybe_fatal(
            unwrap_lock(saver.lock(), "save channel").send(()),
            "Failed to send on save channel",
        );
        vec![]
    }
}

pub fn set_slow_memory(
    settings: Arc<Mutex<Gpu>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::Bool(memory_is_slow)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "gpu");
            settings_lock.slow_memory = *memory_is_slow;
            unwrap_maybe_fatal(
                unwrap_lock(saver.lock(), "save channel").send(()),
                "Failed to send on save channel",
            );
            super::utility::map_empty_result(
                settings_lock.on_set(),
                settings_lock.slow_memory,
            )
        } else {
            vec!["set_slow_memory missing parameter 0".into()]
        }
    }
}

pub fn get_slow_memory(
    settings: Arc<Mutex<Gpu>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "cpu");
        vec![settings_lock.slow_memory.into()]
    }
}
