use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

use super::handler::{ApiMessage, BatteryMessage};

/// Current current (ha!) web method
pub fn current_now(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |val: Option<f64>| tx.send(val).expect("current_now callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Battery(BatteryMessage::ReadCurrentNow(
                    Box::new(callback),
                )))
                .expect("current_now send failed");
            rx.recv().expect("current_now callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| super::utility::map_optional_result(Ok(result)),
    }
}

/// Charge now web method
pub fn charge_now(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |val: Option<f64>| tx.send(val).expect("charge_now callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Battery(BatteryMessage::ReadChargeNow(
                    Box::new(callback),
                )))
                .expect("charge_now send failed");
            rx.recv().expect("charge_now callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| super::utility::map_optional_result(Ok(result)),
    }
}

/// Charge full web method
pub fn charge_full(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |val: Option<f64>| tx.send(val).expect("charge_full callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Battery(BatteryMessage::ReadChargeFull(
                    Box::new(callback),
                )))
                .expect("charge_full send failed");
            rx.recv().expect("charge_full callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| super::utility::map_optional_result(Ok(result)),
    }
}

/// Charge design web method
pub fn charge_design(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |val: Option<f64>| tx.send(val).expect("charge_design callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Battery(BatteryMessage::ReadChargeDesign(
                    Box::new(callback),
                )))
                .expect("charge_design send failed");
            rx.recv().expect("charge_design callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| super::utility::map_optional_result(Ok(result)),
    }
}

/// Charge wattage web method
pub fn charge_power(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |val: Option<f64>| tx.send(val).expect("power_now callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Battery(BatteryMessage::ReadChargePower(
                    Box::new(callback),
                )))
                .expect("power_now send failed");
            rx.recv().expect("power_now callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| super::utility::map_optional_result(Ok(result)),
    }
}

/// Generate set battery charge rate web method
pub fn set_charge_rate(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |rate: f64| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeRate(Some(
                rate as u64,
            ))))
            .expect("set_charge_rate send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(new_val)) = params_in.get(0) {
            setter(new_val);
            vec![(new_val).into()]
        } else {
            vec!["set_charge_rate missing parameter".into()]
        }
    }
}

/// Generate get battery charge rate web method
pub fn get_charge_rate(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback =
            move |rate: Option<u64>| tx.send(rate).expect("get_charge_rate callback send failed");
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::GetChargeRate(
                Box::new(callback),
            )))
            .expect("get_charge_rate send failed");
        rx.recv().expect("get_charge_rate callback recv failed")
    };
    move |_: super::ApiParameterType| vec![getter().map(|x| x.into()).unwrap_or(Primitive::Empty)]
}

/// Generate unset battery charge rate web method
pub fn unset_charge_rate(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move || {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeRate(None)))
            .expect("unset_charge_rate send failed")
    };
    move |_params_in: super::ApiParameterType| {
        setter();
        vec![true.into()]
    }
}

/// Generate set battery charge mode web method
pub fn set_charge_mode(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |mode: String| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeMode(Some(
                mode,
            ))))
            .expect("set_charge_mode send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::String(new_val)) = params_in.get(0) {
            setter(new_val.to_owned());
            vec![new_val.to_owned().into()]
        } else {
            vec!["set_charge_rate missing parameter".into()]
        }
    }
}

/// Generate get battery charge mode web method
pub fn get_charge_mode(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback = move |mode: Option<String>| {
            tx.send(mode).expect("get_charge_mode callback send failed")
        };
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::GetChargeMode(
                Box::new(callback),
            )))
            .expect("get_charge_mode send failed");
        rx.recv().expect("get_charge_mode callback recv failed")
    };
    move |_: super::ApiParameterType| vec![getter().map(|x| x.into()).unwrap_or(Primitive::Empty)]
}

/// Generate unset battery charge mode web method
pub fn unset_charge_mode(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move || {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeMode(None)))
            .expect("unset_charge_mode send failed")
    };
    move |_params_in: super::ApiParameterType| {
        setter();
        vec![true.into()]
    }
}

/// Generate unplugged event receiver web method
pub fn on_unplugged(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    move |_| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::OnUnplugged)
            .expect("on_unplugged send failed");
        vec![true.into()]
    }
}

/// Generate plugged in event receiver web method
pub fn on_plugged(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    move |_| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::OnPluggedIn)
            .expect("on_plugged send failed");
        vec![true.into()]
    }
}

/// Generate set battery charge limit web method
pub fn set_charge_limit(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |limit: f64| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeLimit(Some(
                limit,
            ))))
            .expect("set_charge_limit send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::F64(new_val)) = params_in.get(0) {
            setter(new_val);
            vec![new_val.into()]
        } else {
            vec!["set_charge_limit missing parameter".into()]
        }
    }
}

/// Generate unset battery charge limit web method
pub fn unset_charge_limit(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let unsetter = move || {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeLimit(None)))
            .expect("unset_charge_limit send failed")
    };
    move |_: super::ApiParameterType| {
        unsetter();
        vec![true.into()]
    }
}

/// Charge design web method
pub fn get_charge_limit(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |val: Option<f64>| {
                tx.send(val).expect("get_charge_limit callback send failed")
            };
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::Battery(BatteryMessage::GetChargeLimit(
                    Box::new(callback),
                )))
                .expect("get_charge_limit send failed");
            rx.recv().expect("get_charge_limit callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| super::utility::map_optional_result(Ok(result)),
    }
}
