use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

use crate::settings::{MinMax, SettingError, SettingVariant};
//use crate::utility::{unwrap_lock, unwrap_maybe_fatal};
use super::handler::{ApiMessage, CpuMessage};
use super::utility::map_optional;

/// Available CPUs web method
pub fn max_cpus(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(
        crate::settings::steam_deck::Cpus::cpu_count()
            .map(|x| x as u64)
            .ok_or_else(|| SettingError {
                msg: "Failed to parse CPU count".to_owned(),
                setting: SettingVariant::Cpu,
            }),
    )
}

/// Generate set CPU online web method
pub fn set_cpu_online(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |index: usize, value: bool| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::SetCpuOnline(index, value)))
            .expect("set_cpu_online send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(index)) = params_in.get(0) {
            //let mut settings_lock = unwrap_lock(settings.lock(), "cpu");
            if let Some(&Primitive::Bool(online)) = params_in.get(1) {
                setter(index as usize, online);
                vec![online.into()]
            } else {
                vec!["set_cpu_online missing parameter 1".into()]
            }
        } else {
            vec!["set_cpu_online missing parameter 0".into()]
        }
    }
}

pub fn set_cpus_online(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |values: Vec<bool>| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::SetCpusOnline(values)))
            .expect("set_cpus_online send failed")
    };
    move |params_in: super::ApiParameterType| {
        let mut result = Vec::with_capacity(params_in.len());
        let mut values = Vec::with_capacity(params_in.len());
        for i in 0..params_in.len() {
            if let Primitive::Bool(online) = params_in[i] {
                values.push(online);
                result.push(online.into());
            } else {
                values.push(true);
                result.push(format!("Invalid parameter {}", i).into())
            }
        }
        setter(values);
        result
    }
}

/*pub fn get_cpus_online(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback = move |values: Vec<bool>| tx.send(values).expect("get_cpus_online callback send failed");
        sender.lock().unwrap().send(ApiMessage::Cpu(CpuMessage::GetCpusOnline(Box::new(callback)))).expect("get_cpus_online send failed");
        rx.recv().expect("get_cpus_online callback recv failed")
    };
    move |_: super::ApiParameterType| {
        let result = getter();
        let mut output = Vec::with_capacity(result.len());
        for &status in result.as_slice() {
            output.push(status.into());
        }
        output
    }
}*/

pub fn set_smt(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move |smt: bool| {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |values: Vec<bool>| tx.send(values).expect("set_smt callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Cpu(CpuMessage::SetSmt(smt, Box::new(callback))))
                .expect("set_smt send failed");
            rx.recv().expect("set_smt callback recv failed")
        }
    };
    super::async_utils::AsyncIsh {
        trans_setter: |params| {
            if let Some(&Primitive::Bool(smt_value)) = params.get(0) {
                Ok(smt_value)
            } else {
                Err("set_smt missing/invalid parameter 0".to_owned())
            }
        },
        set_get: getter,
        trans_getter: |result| {
            let mut output = Vec::with_capacity(result.len());
            for &status in result.as_slice() {
                output.push(status.into());
            }
            output
        },
    }
}

pub fn get_smt(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |value: bool| tx.send(value).expect("get_smt callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Cpu(CpuMessage::GetSmt(Box::new(callback))))
                .expect("get_smt send failed");
            rx.recv().expect("get_smt callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| vec![result.into()],
    }
}

pub fn get_cpus_online(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |values: Vec<bool>| {
                tx.send(values)
                    .expect("get_cpus_online callback send failed")
            };
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Cpu(CpuMessage::GetCpusOnline(Box::new(
                    callback,
                ))))
                .expect("get_cpus_online send failed");
            rx.recv().expect("get_cpus_online callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| {
            let mut output = Vec::with_capacity(result.len());
            for &status in result.as_slice() {
                output.push(status.into());
            }
            output
        },
    }
}

pub fn set_clock_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |index: usize, value: MinMax<u64>| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::SetClockLimits(
                index,
                Some(value),
            )))
            .expect("set_clock_limits send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(index)) = params_in.get(0) {
            if let Some(&Primitive::F64(min)) = params_in.get(1) {
                if let Some(&Primitive::F64(max)) = params_in.get(2) {
                    let safe_max = if max < min { min } else { max };
                    let safe_min = if min > max { max } else { min };
                    setter(
                        index as usize,
                        MinMax {
                            min: Some(safe_min as u64),
                            max: Some(safe_max as u64),
                        },
                    );
                    vec![safe_min.into(), safe_max.into()]
                } else {
                    vec!["set_clock_limits missing parameter 2".into()]
                }
            } else {
                vec!["set_clock_limits missing parameter 1".into()]
            }
        } else {
            vec!["set_clock_limits missing parameter 0".into()]
        }
    }
    // TODO allow param 0 and/or 1 to be Primitive::Empty
}

pub fn get_clock_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move |index: usize| {
        let (tx, rx) = mpsc::channel();
        let callback = move |values: Option<MinMax<u64>>| {
            tx.send(values)
                .expect("get_clock_limits callback send failed")
        };
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::GetClockLimits(
                index,
                Box::new(callback),
            )))
            .expect("get_clock_limits send failed");
        rx.recv().expect("get_clock_limits callback recv failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(index)) = params_in.get(0) {
            if let Some(min_max) = getter(index as usize) {
                vec![map_optional(min_max.min), map_optional(min_max.max)]
            } else {
                vec![Primitive::Empty, Primitive::Empty]
            }
        } else {
            vec!["get_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn unset_clock_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |index: usize| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::SetClockLimits(index, None)))
            .expect("unset_clock_limits send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(index)) = params_in.get(0) {
            setter(index as usize);
            vec![true.into()]
        } else {
            vec!["get_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn set_cpu_governor(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |index: usize, governor: String| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::SetCpuGovernor(index, governor)))
            .expect("set_cpu_governor send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(index)) = params_in.get(0) {
            if let Some(Primitive::String(governor)) = params_in.get(1) {
                setter(index as usize, governor.to_owned());
                vec![(governor as &str).into()]
            } else {
                vec!["set_cpu_governor missing parameter 1".into()]
            }
        } else {
            vec!["set_cpu_governor missing parameter 0".into()]
        }
    }
}

pub fn set_cpus_governors(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |governors: Vec<String>| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::SetCpusGovernor(governors)))
            .expect("set_cpus_governor send failed")
    };
    move |params_in: super::ApiParameterType| {
        let mut result = Vec::with_capacity(params_in.len());
        let mut values = Vec::with_capacity(params_in.len());
        for i in 0..params_in.len() {
            if let Primitive::String(gov) = &params_in[i] {
                values.push(gov.to_owned());
                result.push((gov as &str).into());
            } else {
                //values.push(true);
                result.push(format!("Invalid parameter {}", i).into())
            }
        }
        setter(values);
        result
    }
}

pub fn get_cpu_governors(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback = move |values: Vec<String>| {
            tx.send(values)
                .expect("get_cpu_governors callback send failed")
        };
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Cpu(CpuMessage::GetCpusGovernor(Box::new(
                callback,
            ))))
            .expect("get_cpu_governors send failed");
        rx.recv().expect("get_cpu_governors callback recv failed")
    };
    move |_: super::ApiParameterType| {
        let result = getter();
        let mut output = Vec::with_capacity(result.len());
        for cpu in result.as_slice() {
            output.push(cpu.clone().into());
        }
        output
    }
}
