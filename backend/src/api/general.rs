use std::sync::mpsc::{Sender, self};
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
    let setter = move |pers: bool|
        sender.lock()
            .unwrap()
            .send(ApiMessage::General(GeneralMessage::SetPersistent(pers))).expect("set_persistent send failed");
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
        let callback = move |value: bool| tx.send(value).expect("get_persistent callback send failed");
        sender.lock().unwrap().send(ApiMessage::General(GeneralMessage::GetPersistent(Box::new(callback)))).expect("get_persistent send failed");
        rx.recv().expect("get_persistent callback recv failed")
    };
    move |_: super::ApiParameterType| {
        vec![getter().into()]
    }
}

/// Generate load app settings from file web method
pub fn load_settings(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move |path: String, name: String|
        sender.lock()
            .unwrap()
            .send(ApiMessage::LoadSettings(path, name)).expect("load_settings send failed");
    move |params_in: super::ApiParameterType| {
        if let Some(Primitive::String(path)) = params_in.get(0) {
            if let Some(Primitive::String(name)) = params_in.get(1) {
                setter(path.to_owned(), name.to_owned());
                vec![true.into()]
            } else {
                vec!["load_settings missing name parameter".into()]
            }
            //let mut general_lock = unwrap_lock(settings.general.lock(), "general");
        } else {
            vec!["load_settings missing path parameter".into()]
        }
    }
}

/// Generate load default settings from file web method
pub fn load_default_settings(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let setter = move ||
        sender.lock()
            .unwrap()
            .send(ApiMessage::LoadMainSettings).expect("load_default_settings send failed");
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
    let setter = move ||
        sender.lock()
            .unwrap()
            .send(ApiMessage::LoadSystemSettings).expect("load_default_settings send failed");
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
pub fn get_name(
    sender: Sender<ApiMessage>,
) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |name: String| tx.send(name).expect("get_name callback send failed");
            sender2.lock().unwrap().send(ApiMessage::General(GeneralMessage::GetCurrentProfileName(Box::new(callback)))).expect("get_name send failed");
            rx.recv().expect("get_name callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |result| {
            vec![result.into()]
        }
    }
}

/// Generate wait for all locks to be available web method
pub fn lock_unlock_all(
    sender: Sender<ApiMessage>,
) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move || {
            let (tx, rx) = mpsc::channel();
            let callback = move |x| tx.send(x).expect("lock_unlock_all callback send failed");
            sender2.lock().unwrap().send(ApiMessage::WaitForEmptyQueue(Box::new(callback))).expect("lock_unlock_all send failed");
            rx.recv().expect("lock_unlock_all callback recv failed")
        }
    };
    super::async_utils::AsyncIshGetter {
        set_get: getter,
        trans_getter: |_| {
            vec![true.into()]
        }
    }
}

/// Generate get limits web method
pub fn get_limits(
    sender: Sender<ApiMessage>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let sender = Mutex::new(sender); // Sender is not Sync; this is required for safety
    let getter = move || {
        let (tx, rx) = mpsc::channel();
        let callback = move |value: super::SettingsLimits| tx.send(value).expect("get_limits callback send failed");
        sender.lock().unwrap().send(ApiMessage::GetLimits(Box::new(callback))).expect("get_limits send failed");
        rx.recv().expect("get_limits callback recv failed")
    };
    move |_: super::ApiParameterType| {
        vec![Primitive::Json(serde_json::to_string(&getter()).unwrap())]
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
