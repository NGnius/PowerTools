use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

//use crate::utility::{unwrap_lock, unwrap_maybe_fatal};
use super::handler::{ApiMessage, GeneralMessage};

/// Generate set persistent web method
pub fn set_persistent(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |pers: bool| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::General(GeneralMessage::SetPersistent(pers)))
            .expect("set_persistent send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(&Primitive::Bool(new_val)) = params_in.get(0) {
            setter(new_val);
            //log::debug!("Persistent is now {}", settings_lock.persistent);
            vec![new_val.into()]
        } else {
            vec!["set_persistent missing parameter".into()]
        }
    }
}

/// Generate get persistent save mode web method
pub fn get_persistent(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback =
            move |value: bool| tx.send(value).expect("get_persistent callback send failed");
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::General(GeneralMessage::GetPersistent(
                Box::new(callback),
            )))
            .expect("get_persistent send failed");
        rx.recv().expect("get_persistent callback recv failed")
    };
    move |_: super::ApiParameterType| vec![getter().into()]
}

/// Generate load app settings from file web method
pub fn load_settings(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |path: u64, name: String| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::LoadSettings(path, name))
            .expect("load_settings send failed")
    };
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::String(id)) = params_in.get(0) {
            if let Some(Primitive::String(name)) = params_in.get(1) {
                setter(id.parse().unwrap_or_default(), name.to_owned());
                vec![true.into()]
            } else {
                log::warn!("load_settings missing name parameter");
                vec!["load_settings missing name parameter".into()]
            }
            //let mut general_lock = unwrap_lock(settings.general.lock(), "general");
        } else {
            log::warn!("load_settings missing id parameter");
            vec!["load_settings missing id parameter".into()]
        }
    }
}

/// Generate load default settings from file web method
pub fn load_default_settings(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move || {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::LoadMainSettings)
            .expect("load_default_settings send failed")
    };
    move |_: super::ApiParameterType| {
        setter();
        vec![true.into()]
        /*match settings.load_file(
                crate::consts::DEFAULT_SETTINGS_FILE.into(),
                crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
                true
            ) {
            Err(e) => vec![e.msg.into()],
            Ok(success) => super::utility::map_empty_result(
                            settings.clone().on_set(),
                            success
                        )
        }*/
    }
}

/// Generate load system default settings from file web method
pub fn load_system_settings(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move || {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::LoadSystemSettings)
            .expect("load_default_settings send failed")
    };
    move |_: super::ApiParameterType| {
        setter();
        vec![true.into()]
        /*match settings.load_file(
                crate::consts::DEFAULT_SETTINGS_FILE.into(),
                crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
                true
            ) {
            Err(e) => vec![e.msg.into()],
            Ok(success) => super::utility::map_empty_result(
                            settings.clone().on_set(),
                            success
                        )
        }*/
    }
}

/// Generate get current settings name
pub fn get_name(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback =
                move |name: String| tx.send(name).expect("get_name callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::General(GeneralMessage::GetCurrentProfileName(
                    Box::new(callback),
                )))
                .expect("get_name send failed");
            rx.recv().expect("get_name callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| vec![result.into()],
    }
}

/// Generate get current settings name
pub fn get_path(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |name: std::path::PathBuf| {
                tx.send(name).expect("get_path callback send failed")
            };
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::General(GeneralMessage::GetPath(Box::new(
                    callback,
                ))))
                .expect("get_name send failed");
            rx.recv().expect("get_path callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| vec![result.to_string_lossy().to_string().into()],
    }
}

/// Generate wait for all locks to be available web method
pub fn lock_unlock_all(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |x| tx.send(x).expect("lock_unlock_all callback send failed");
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::WaitForEmptyQueue(Box::new(callback)))
                .expect("lock_unlock_all send failed");
            rx.recv().expect("lock_unlock_all callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |_| vec![true.into()],
    }
}

/// Generate get limits web method
pub fn get_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback = move |value: super::SettingsLimits| {
            tx.send(value).expect("get_limits callback send failed")
        };
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::GetLimits(Box::new(callback)))
            .expect("get_limits send failed");
        rx.recv().expect("get_limits callback recv failed")
    };
    move |_: super::ApiParameterType| {
        vec![Primitive::Json(serde_json::to_string(&getter()).unwrap())]
    }
}

/// Generate get current driver name
pub fn get_provider(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move |provider_name: String| {
            let (tx, rx) = mpsc::channel();
            let callback = move |name: crate::persist::DriverJson| {
                tx.send(name).expect("get_provider callback send failed")
            };
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::GetProvider(provider_name, Box::new(callback)))
                .expect("get_provider send failed");
            rx.recv().expect("get_provider callback recv failed")
        }
    };
    super::async_utils::AsyncIsh {
        trans_setter: |mut params| {
            if let Some(Primitive::String(name)) = params.pop() {
                Ok(name.to_owned())
            } else {
                Err(format!("Invalid/missing single param in get_provider"))
            }
        },
        set_get: getter,
        trans_getter: |result| vec![format!("{:?}", result).into()],
    }
}

pub fn gunter(_: super::ApiParameterType) -> super::ApiParameterType {
    std::thread::spawn(|| {
        log::info!("Zhu Li, do the thing!");
        crate::settings::driver::maybe_do_button();
        log::info!("Thing done.")
    });
    vec![true.into()]
}

/// API web method to send log messages to the back-end log, callable from the front-end
pub fn log_it() -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |params| {
        if let Some(Primitive::F64(level)) = params.get(0) {
            if let Some(Primitive::String(msg)) = params.get(1) {
                log_msg_by_level(*level as u8, msg);
                vec![true.into()]
            } else if let Some(Primitive::Json(msg)) = params.get(1) {
                log_msg_by_level(*level as u8, msg);
                vec![true.into()]
            } else {
                log::warn!("Got log_it call with wrong/missing 2nd parameter");
                vec![false.into()]
            }
        } else {
            log::warn!("Got log_it call with wrong/missing 1st parameter");
            vec![false.into()]
        }
    }
}

fn log_msg_by_level(level: u8, msg: &str) {
    match level {
        1 => log::trace!("FRONT-END: {}", msg),
        2 => log::debug!("FRONT-END: {}", msg),
        3 => log::info!("FRONT-END: {}", msg),
        4 => log::warn!("FRONT-END: {}", msg),
        5 => log::error!("FRONT-END: {}", msg),
        _ => log::trace!("FRONT-END: {}", msg),
    }
}

/// Generate set battery charge rate web method
pub fn force_apply(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |_: ()| {
        sender
            .lock()
            .unwrap()
            .send(ApiMessage::General(GeneralMessage::ApplyNow))
            .expect("force_apply send failed")
    };
    move |_params_in: super::ApiParameterType| {
        setter(());
        vec![true.into()]
    }
}

/// Generate get periodicals aggregate method
pub fn get_periodicals(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (rx_curr, callback_curr) = build_comms("battery current callback send failed");
            let (rx_charge_now, callback_charge_now) = build_comms("battery charge now callback send failed");
            let (rx_charge_full, callback_charge_full) = build_comms("battery charge full callback send failed");
            let (rx_charge_power, callback_charge_power) = build_comms("battery charge power callback send failed");

            let (rx_path, callback_path) = build_comms("general get path (periodical) send failed");

            let sender_locked = sender2
                .lock()
                .unwrap();
            let curr = wait_for_response(&*sender_locked, rx_curr,
                    ApiMessage::Battery(super::handler::BatteryMessage::ReadCurrentNow(callback_curr)), "battery current");
            let charge_now = wait_for_response(&*sender_locked, rx_charge_now,
                    ApiMessage::Battery(super::handler::BatteryMessage::ReadChargeNow(callback_charge_now)), "battery charge now");
            let charge_full = wait_for_response(&*sender_locked, rx_charge_full,
                    ApiMessage::Battery(super::handler::BatteryMessage::ReadChargeFull(callback_charge_full)), "battery charge full");
            let charge_power = wait_for_response(&*sender_locked, rx_charge_power,
                    ApiMessage::Battery(super::handler::BatteryMessage::ReadChargePower(callback_charge_power)), "battery charge power");

            let settings_path = wait_for_response(&*sender_locked, rx_path,
                    ApiMessage::General(GeneralMessage::GetPath(callback_path)), "general get path");
            vec![
                super::utility::map_optional(curr),
                super::utility::map_optional(charge_now),
                super::utility::map_optional(charge_full),
                super::utility::map_optional(charge_power),

                super::utility::map_optional(settings_path.to_str()),
            ]
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| result,
    }
}

fn build_comms<'a, T: Send + 'a>(msg: &'static str) -> (mpsc::Receiver<T>, Box<dyn FnOnce(T) + Send + 'a>) {
    let (tx, rx) = mpsc::channel();
    let callback = move |t: T| tx.send(t).expect(msg);
    (rx, Box::new(callback))
}

fn wait_for_response<T>(sender: &Sender<ApiMessage>, rx: mpsc::Receiver<T>, api_msg: ApiMessage, op: &str) -> T {
    sender.send(api_msg).expect(&format!("{} send failed", op));
    rx.recv().expect(&format!("{} callback recv failed", op))
}
