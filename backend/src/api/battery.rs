use std::sync::mpsc::{Sender, self};
use std::sync::Mutex;
use usdpl_back::core::serdes::Primitive;

use super::handler::{ApiMessage, BatteryMessage};

/// Current current (ha!) web method
pub fn current_now(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(crate::settings::Battery::read_current_now())
}

/// Charge now web method
pub fn charge_now(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(crate::settings::Battery::read_charge_now())
}

/// Charge full web method
pub fn charge_full(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(crate::settings::Battery::read_charge_full())
}

/// Charge design web method
pub fn charge_design(_: super::ApiParameterType) -> super::ApiParameterType {
    super::utility::map_result(crate::settings::Battery::read_charge_design())
}

/// Generate set battery charge rate web method
pub fn set_charge_rate(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |rate: f64|
        sender.lock()
            .unwrap()
            .send(ApiMessage::Battery(BatteryMessage::SetChargeRate(Some(rate as u64))))
            .expect("set_charge_rate send failed");
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
        let callback = move |rate: Option<u64>| tx.send(rate).expect("get_charge_rate callback send failed");
        sender.lock().unwrap().send(ApiMessage::Battery(BatteryMessage::GetChargeRate(Box::new(callback)))).expect("get_charge_rate send failed");
        rx.recv().expect("get_charge_rate callback recv failed")
    };
    move |_: super::ApiParameterType| {
        vec![getter().map(|x| x.into()).unwrap_or(Primitive::Empty)]
    }
}

/// Generate unset battery charge rate web method
pub fn unset_charge_rate(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move || sender.lock().unwrap().send(ApiMessage::Battery(BatteryMessage::SetChargeRate(None))).expect("unset_charge_rate send failed");
    move |_params_in: super::ApiParameterType| {
        setter();
        vec![true.into()]
    }
}
