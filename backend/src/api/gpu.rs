use std::sync::mpsc::{Sender, self};
use std::sync::{Mutex, Arc};
use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

use crate::settings::MinMax;
//use crate::utility::{unwrap_lock, unwrap_maybe_fatal};
use super::handler::{ApiMessage, GpuMessage};

pub fn set_ppt(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |fast: u64, slow: u64|
        sender.lock()
            .unwrap()
            .send(ApiMessage::Gpu(GpuMessage::SetPpt(Some(fast), Some(slow)))).expect("set_ppt send failed");
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(fast_ppt)) = params_in.get(0) {
            if let Some(&Primitive::F64(slow_ppt)) = params_in.get(1) {
                setter(fast_ppt as u64, slow_ppt as u64);
                vec![(fast_ppt as u64).into(), (slow_ppt as u64).into()]
            } else {
                vec!["set_ppt missing parameter 1".into()]
            }
        } else {
            vec!["set_ppt missing parameter 0".into()]
        }
    }
}

pub fn get_ppt(
    sender: Sender<ApiMessage>,
) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |ppt: (Option<u64>, Option<u64>)| tx.send(ppt).expect("get_ppt callback send failed");
            sender2.lock().unwrap().send(ApiMessage::Gpu(GpuMessage::GetPpt(Box::new(callback)))).expect("get_ppt send failed");
            rx.recv().expect("get_ppt callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |(fast, slow): (Option<u64>, Option<u64>)| {
            vec![
                fast.map(|x| x.into()).unwrap_or(Primitive::Empty),
                slow.map(|x| x.into()).unwrap_or(Primitive::Empty),
            ]
        }
    }
}

pub fn unset_ppt(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move ||
        sender.lock()
            .unwrap()
            .send(ApiMessage::Gpu(GpuMessage::SetPpt(None, None))).expect("set_ppt send failed");
    move |_: super::ApiParameterType| {
        setter();
        vec![true.into()]
    }
}

pub fn set_clock_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |value: MinMax<u64>|
        sender.lock()
            .unwrap()
            .send(ApiMessage::Gpu(GpuMessage::SetClockLimits(Some(value)))).expect("set_clock_limits send failed");
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(min)) = params_in.get(0) {
            if let Some(&Primitive::F64(max)) = params_in.get(1) {
                let safe_max = if max < min {
                    min
                } else {
                    max
                };
                let safe_min = if min > max {
                    max
                } else {
                    min
                };
                setter(MinMax {
                    min: safe_min as _,
                    max: safe_max as _,
                });
                vec![(safe_min as u64).into(), (safe_max as u64).into()]
            } else {
                vec!["set_clock_limits missing parameter 1".into()]
            }
        } else {
            vec!["set_clock_limits missing parameter 0".into()]
        }
    }
}

pub fn get_clock_limits(
    sender: Sender<ApiMessage>,
) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move|| {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |clocks: Option<MinMax<u64>>| tx.send(clocks).expect("get_clock_limits callback send failed");
            sender2.lock().unwrap().send(ApiMessage::Gpu(GpuMessage::GetClockLimits(Box::new(callback)))).expect("get_clock_limits send failed");
            rx.recv().expect("get_clock_limits callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |clocks: Option<MinMax<u64>>| {
            clocks.map(|x| vec![
                x.min.into(), x.max.into()
            ]).unwrap_or_else(|| vec![Primitive::Empty, Primitive::Empty])
        }
    }
}

pub fn unset_clock_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move ||
        sender.lock()
            .unwrap()
            .send(ApiMessage::Gpu(GpuMessage::SetClockLimits(None))).expect("unset_clock_limits send failed");
    move |_: super::ApiParameterType| {
        setter();
        vec![true.into()]
    }
}

pub fn set_slow_memory(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |value: bool|
        sender.lock()
            .unwrap()
            .send(ApiMessage::Gpu(GpuMessage::SetSlowMemory(value))).expect("unset_clock_limits send failed");
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::Bool(memory_is_slow)) = params_in.get(0) {
            setter(memory_is_slow);
            vec![memory_is_slow.into()]
        } else {
            vec!["set_slow_memory missing parameter 0".into()]
        }
    }
}

pub fn get_slow_memory(
    sender: Sender<ApiMessage>,
) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |value: bool| tx.send(value).expect("get_slow_memory callback send failed");
            sender2.lock().unwrap().send(ApiMessage::Gpu(GpuMessage::GetSlowMemory(Box::new(callback)))).expect("get_slow_memory send failed");
            rx.recv().expect("get_slow_memory callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |value: bool| {
            vec![value.into()]
        }
    }
}
