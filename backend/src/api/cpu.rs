use std::sync::{mpsc::Sender, Arc, Mutex};
use usdpl_back::core::serdes::Primitive;

use crate::settings::{Cpu, OnSet, SettingError, SettingVariant, MinMax};
use crate::utility::{unwrap_lock, unwrap_maybe_fatal};

/// Available CPUs web method
pub fn max_cpus(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(
        Cpu::cpu_count()
            .map(|x| x as u64)
            .ok_or_else(
                || SettingError {
                    msg: "Failed to parse CPU count".to_owned(),
                    setting: SettingVariant::Cpu,
                    }
            )
        )
}

/// Generate set CPU online web method
pub fn set_cpu_online(
    settings: Arc<Mutex<Vec<Cpu>>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(index)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "cpu");
            if let Some(cpu) = settings_lock.get_mut(*index as usize) {
                if let Some(Primitive::Bool(online)) = params_in.get(1) {
                    cpu.online = *online;
                    unwrap_maybe_fatal(
                        unwrap_lock(saver.lock(), "save channel").send(()),
                        "Failed to send on save channel",
                    );
                    super::utility::map_empty_result(
                        cpu.on_set(),
                        cpu.online,
                    )
                } else {
                    vec!["set_cpu_online missing parameter 1".into()]
                }
            } else {
                vec!["set_cpu_online cpu index out of bounds".into()]
            }
        } else {
            vec!["set_cpu_online missing parameter 0".into()]
        }
    }
}

pub fn get_cpus_online(
    settings: Arc<Mutex<Vec<Cpu>>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "cpu");
        let mut output = Vec::with_capacity(settings_lock.len());
        for cpu in settings_lock.as_slice() {
            output.push(cpu.online.into());
        }
        output
    }
}

pub fn set_clock_limits(
    settings: Arc<Mutex<Vec<Cpu>>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(index)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "cpu");
            if let Some(cpu) = settings_lock.get_mut(*index as usize) {
                if let Some(Primitive::F64(min)) = params_in.get(1) {
                    if let Some(Primitive::F64(max)) = params_in.get(2) {
                        cpu.clock_limits = Some(MinMax {
                            min: *min as _,
                            max: *max as _,
                        });
                        unwrap_maybe_fatal(
                            unwrap_lock(saver.lock(), "save channel").send(()),
                            "Failed to send on save channel",
                        );
                        match cpu.on_set() {
                            Ok(_) => vec![
                                cpu.clock_limits.as_ref().unwrap().min.into(),
                                cpu.clock_limits.as_ref().unwrap().max.into(),
                            ],
                            Err(e) => vec![e.msg.into()]
                        }
                    } else {
                        vec!["set_clock_limits missing parameter 2".into()]
                    }
                } else {
                    vec!["set_clock_limits missing parameter 1".into()]
                }
            } else {
                vec!["set_clock_limits cpu index out of bounds".into()]
            }
        } else {
            vec!["set_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn get_clock_limits(
    settings: Arc<Mutex<Vec<Cpu>>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(index)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "cpu");
            if let Some(cpu) = settings_lock.get_mut(*index as usize) {
                if let Some(min_max) = &cpu.clock_limits {
                    vec![min_max.max.into(), min_max.min.into()]
                } else {
                    vec![Primitive::Empty, Primitive::Empty]
                }
            } else {
                vec!["get_clock_limits cpu index out of bounds".into()]
            }
        } else {
            vec!["get_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn unset_clock_limits(
    settings: Arc<Mutex<Vec<Cpu>>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(index)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "cpu");
            if let Some(cpu) = settings_lock.get_mut(*index as usize) {
                cpu.clock_limits = None;
                unwrap_maybe_fatal(
                    unwrap_lock(saver.lock(), "save channel").send(()),
                    "Failed to send on save channel",
                );
                super::utility::map_empty_result(cpu.on_set(), true)
            } else {
                vec!["get_clock_limits cpu index out of bounds".into()]
            }
        } else {
            vec!["get_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn set_cpu_governor(
    settings: Arc<Mutex<Vec<Cpu>>>,
    saver: Sender<()>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let saver = Mutex::new(saver); // Sender is not Sync; this is required for safety
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::F64(index)) = params_in.get(0) {
            let mut settings_lock = unwrap_lock(settings.lock(), "cpu");
            if let Some(cpu) = settings_lock.get_mut(*index as usize) {
                if let Some(Primitive::String(governor)) = params_in.get(1) {
                    cpu.governor = governor.to_owned();
                    unwrap_maybe_fatal(
                        unwrap_lock(saver.lock(), "save channel").send(()),
                        "Failed to send on save channel",
                    );
                    super::utility::map_empty_result(
                        cpu.on_set(),
                        &cpu.governor as &str,
                    )
                } else {
                    vec!["set_cpu_governor missing parameter 1".into()]
                }
            } else {
                vec!["set_cpu_governor cpu index out of bounds".into()]
            }
        } else {
            vec!["set_cpu_governor missing parameter 0".into()]
        }
    }
}

pub fn get_cpu_governors(
    settings: Arc<Mutex<Vec<Cpu>>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let settings_lock = unwrap_lock(settings.lock(), "cpu");
        let mut output = Vec::with_capacity(settings_lock.len());
        for cpu in settings_lock.as_slice() {
            output.push(cpu.governor.clone().into());
        }
        output
    }
}
