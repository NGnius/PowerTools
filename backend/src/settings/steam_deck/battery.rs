use std::convert::Into;

use super::oc_limits::{BatteryLimits, OverclockLimits};
use super::util::ChargeMode;
use crate::api::RangeLimit;
use crate::persist::{BatteryEventJson, BatteryJson};
use crate::settings::TBattery;
use crate::settings::{OnPowerEvent, OnResume, OnSet, PowerMode, SettingError};

#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate: Option<u64>,
    pub charge_mode: Option<ChargeMode>,
    events: Vec<EventInstruction>,
    limits: BatteryLimits,
    state: crate::state::steam_deck::Battery,
    driver_mode: crate::persist::DriverJson,
}

#[derive(Debug, Clone)]
enum EventTrigger {
    PluggedIn,
    PluggedOut,
    BatteryAbove(f64),
    BatteryBelow(f64),
    Ignored,
}

#[derive(Debug, Clone)]
struct EventInstruction {
    trigger: EventTrigger,
    charge_rate: Option<u64>,
    charge_mode: Option<ChargeMode>,
    is_triggered: bool,
}

impl OnPowerEvent for EventInstruction {
    fn on_power_event(&mut self, new_mode: PowerMode) -> Result<(), Vec<SettingError>> {
        match (&self.trigger, new_mode) {
            (EventTrigger::PluggedIn, PowerMode::PluggedIn) => {
                log::info!("Steam Deck plugged in event handled");
                self.set_all()
            }
            (EventTrigger::PluggedOut, PowerMode::PluggedOut) => {
                log::info!("Steam Deck plugged out event handled");
                self.set_all()
            }
            (EventTrigger::BatteryAbove(exp), PowerMode::BatteryCharge(act)) => {
                if act > *exp {
                    if self.is_triggered {
                        Ok(())
                    } else {
                        self.is_triggered = true;
                        log::info!("Steam Deck battery above {} event handled", exp);
                        self.set_all()
                    }
                } else {
                    self.is_triggered = false;
                    Ok(())
                }
            }
            (EventTrigger::BatteryBelow(exp), PowerMode::BatteryCharge(act)) => {
                if act < *exp {
                    if self.is_triggered {
                        Ok(())
                    } else {
                        self.is_triggered = true;
                        log::info!("Steam Deck battery below {} event handled", exp);
                        self.set_all()
                    }
                } else {
                    self.is_triggered = false;
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl EventInstruction {
    #[inline]
    fn trigger_to_str(mode: EventTrigger) -> String {
        match mode {
            EventTrigger::PluggedIn => "plug-in".to_owned(),
            EventTrigger::PluggedOut => "plug-out".to_owned(),
            EventTrigger::BatteryAbove(x) => format!(">{:#0.2}", x * 100.0),
            EventTrigger::BatteryBelow(x) => format!("<{:#0.2}", x * 100.0),
            EventTrigger::Ignored => "/shrug".to_owned(),
        }
    }

    #[inline]
    fn str_to_trigger(s: &str) -> Option<EventTrigger> {
        match s {
            "plug-in" => Some(EventTrigger::PluggedIn),
            "plug-out" => Some(EventTrigger::PluggedOut),
            s if s.starts_with('>') => s
                .trim_start_matches('>')
                .parse::<f64>()
                .ok()
                .map(|x| EventTrigger::BatteryAbove(x/100.0)),
            s if s.starts_with('<') => s
                .trim_start_matches('<')
                .parse::<f64>()
                .ok()
                .map(|x| EventTrigger::BatteryBelow(x/100.0)),
            _ => None,
        }
    }

    fn from_json(other: BatteryEventJson, _version: u64) -> Self {
        Self {
            trigger: Self::str_to_trigger(&other.trigger).unwrap_or(EventTrigger::Ignored),
            charge_rate: other.charge_rate,
            charge_mode: other
                .charge_mode
                .map(|x| Battery::str_to_charge_mode(&x))
                .flatten(),
            is_triggered: false,
        }
    }

    fn set_charge_mode(&self) -> Result<(), SettingError> {
        if let Some(charge_mode) = self.charge_mode {
            super::util::set(super::util::Setting::ChargeMode, charge_mode as _)
                .map_err(|e| SettingError {
                    msg: format!("Failed to set charge mode: {}", e),
                    setting: crate::settings::SettingVariant::Battery,
                })
                .map(|_| ())
        } else {
            Ok(())
        }
    }

    fn set_charge_rate(&self) -> Result<(), SettingError> {
        if let Some(charge_rate) = self.charge_rate {
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, charge_rate)
                .map_err(|e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: crate::settings::SettingVariant::Battery,
                })
                .map(|_| ())
        } else {
            Ok(())
        }
    }

    fn set_all(&self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        self.set_charge_rate().unwrap_or_else(|e| errors.push(e));
        self.set_charge_mode().unwrap_or_else(|e| errors.push(e));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Into<BatteryEventJson> for EventInstruction {
    fn into(self) -> BatteryEventJson {
        BatteryEventJson {
            trigger: Self::trigger_to_str(self.trigger),
            charge_rate: self.charge_rate,
            charge_mode: self.charge_mode.map(|c| Battery::charge_mode_to_str(c)),
        }
    }
}

const BATTERY_VOLTAGE: f64 = 7.7;

const BATTERY_CHARGE_RATE_PATH: &str = "/sys/class/hwmon/hwmon5/maximum_battery_charge_rate"; // write-only
const BATTERY_CURRENT_NOW_PATH: &str = "/sys/class/power_supply/BAT1/current_now"; // read-only
const BATTERY_CHARGE_NOW_PATH: &str = "/sys/class/power_supply/BAT1/charge_now"; // read-only
const BATTERY_CHARGE_FULL_PATH: &str = "/sys/class/power_supply/BAT1/charge_full"; // read-only
const BATTERY_CHARGE_DESIGN_PATH: &str = "/sys/class/power_supply/BAT1/charge_full_design"; // read-only
const USB_PD_IN_MVOLTAGE_PATH: &str = "/sys/class/hwmon/hwmon5/in0_input"; // read-only

impl Battery {
    #[inline]
    pub fn from_json(other: BatteryJson, version: u64) -> Self {
        let (oc_limits, is_default) = OverclockLimits::load_or_default();
        let oc_limits = oc_limits.battery;
        let driver = if is_default {
            crate::persist::DriverJson::SteamDeck
        } else {
            crate::persist::DriverJson::SteamDeckAdvance
        };
        match version {
            0 => Self {
                charge_rate: other.charge_rate,
                charge_mode: other
                    .charge_mode
                    .map(|x| Self::str_to_charge_mode(&x))
                    .flatten(),
                events: other
                    .events
                    .into_iter()
                    .map(|x| EventInstruction::from_json(x, version))
                    .collect(),
                limits: oc_limits,
                state: crate::state::steam_deck::Battery::default(),
                driver_mode: driver,
            },
            _ => Self {
                charge_rate: other.charge_rate,
                charge_mode: other
                    .charge_mode
                    .map(|x| Self::str_to_charge_mode(&x))
                    .flatten(),
                events: other
                    .events
                    .into_iter()
                    .map(|x| EventInstruction::from_json(x, version))
                    .collect(),
                limits: oc_limits,
                state: crate::state::steam_deck::Battery::default(),
                driver_mode: driver,
            },
        }
    }

    #[inline]
    fn charge_mode_to_str(mode: ChargeMode) -> String {
        match mode {
            ChargeMode::Normal => "normal",
            ChargeMode::Idle => "idle",
            ChargeMode::Discharge => "discharge",
        }
        .to_owned()
    }

    #[inline]
    fn str_to_charge_mode(s: &str) -> Option<ChargeMode> {
        match s {
            "normal" => Some(ChargeMode::Normal),
            "idle" => Some(ChargeMode::Idle),
            "discharge" => Some(ChargeMode::Discharge),
            _ => None,
        }
    }

    fn set_charge_mode(&mut self) -> Result<(), SettingError> {
        if let Some(charge_mode) = self.charge_mode {
            self.state.charge_mode_set = true;
            super::util::set(super::util::Setting::ChargeMode, charge_mode as _)
                .map_err(|e| SettingError {
                    msg: format!("Failed to set charge mode: {}", e),
                    setting: crate::settings::SettingVariant::Battery,
                })
                .map(|_| ())
        } else if self.state.charge_mode_set {
            self.state.charge_mode_set = false;
            super::util::set(super::util::Setting::ChargeMode, ChargeMode::Normal as _)
                .map_err(|e| SettingError {
                    msg: format!("Failed to set charge mode: {}", e),
                    setting: crate::settings::SettingVariant::Battery,
                })
                .map(|_| ())
        } else {
            Ok(())
        }
    }

    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        if let Some(charge_rate) = self.charge_rate {
            self.state.charge_rate_set = true;
            usdpl_back::api::files::write_single(BATTERY_CHARGE_RATE_PATH, charge_rate)
                .map_err(|e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                    setting: crate::settings::SettingVariant::Battery,
                })
                .unwrap_or_else(|e| errors.push(e));
        } else if self.state.charge_rate_set {
            self.state.charge_rate_set = false;
            usdpl_back::api::files::write_single(
                BATTERY_CHARGE_RATE_PATH,
                self.limits.charge_rate.max,
            )
            .map_err(|e| SettingError {
                msg: format!("Failed to write to `{}`: {}", BATTERY_CHARGE_RATE_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            })
            .unwrap_or_else(|e| errors.push(e));
        }
        self.set_charge_mode().unwrap_or_else(|e| errors.push(e));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn clamp_all(&mut self) {
        if let Some(charge_rate) = &mut self.charge_rate {
            *charge_rate =
                (*charge_rate).clamp(self.limits.charge_rate.min, self.limits.charge_rate.max);
        }
    }

    pub fn read_current_now() -> Result<u64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CURRENT_NOW_PATH) {
            Err(e) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CURRENT_NOW_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            // this value is in uA, while it's set in mA
            // so convert this to mA for consistency
            Ok(val) => Ok(val / 1000),
        }
    }

    pub fn read_charge_now() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CHARGE_NOW_PATH) {
            Err(e) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_NOW_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            // convert to Wh
            Ok(val) => Ok((val as f64) / 1000000.0 * BATTERY_VOLTAGE),
        }
    }

    pub fn read_charge_full() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CHARGE_FULL_PATH) {
            Err(e) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", BATTERY_CHARGE_FULL_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            // convert to Wh
            Ok(val) => Ok((val as f64) / 1000000.0 * BATTERY_VOLTAGE),
        }
    }

    pub fn read_charge_design() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(BATTERY_CHARGE_DESIGN_PATH) {
            Err(e) => Err(SettingError {
                msg: format!(
                    "Failed to read from `{}`: {}",
                    BATTERY_CHARGE_DESIGN_PATH, e
                ),
                setting: crate::settings::SettingVariant::Battery,
            }),
            // convert to Wh
            Ok(val) => Ok((val as f64) / 1000000.0 * BATTERY_VOLTAGE),
        }
    }

    pub fn read_usb_voltage() -> Result<f64, SettingError> {
        match usdpl_back::api::files::read_single::<_, u64, _>(USB_PD_IN_MVOLTAGE_PATH) {
            Err(e) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", USB_PD_IN_MVOLTAGE_PATH, e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            // convert to V (from mV)
            Ok(val) => Ok((val as f64) / 1000.0),
        }
    }

    pub fn system_default() -> Self {
        let (oc_limits, is_default) = OverclockLimits::load_or_default();
        let oc_limits = oc_limits.battery;
        let driver = if is_default {
            crate::persist::DriverJson::SteamDeck
        } else {
            crate::persist::DriverJson::SteamDeckAdvance
        };
        Self {
            charge_rate: None,
            charge_mode: None,
            events: Vec::new(),
            limits: oc_limits,
            state: crate::state::steam_deck::Battery::default(),
            driver_mode: driver,
        }
    }

    fn find_limit_event(&self) -> Option<usize> {
        for (i, event) in self.events.iter().enumerate() {
            match event.trigger {
                EventTrigger::BatteryAbove(_) => {
                    if event.charge_mode.is_some() {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_unlimit_event(&self) -> Option<usize> {
        for (i, event) in self.events.iter().enumerate() {
            match event.trigger {
                EventTrigger::BatteryBelow(_) => {
                    if event.charge_mode.is_some() {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: self.charge_rate,
            charge_mode: self.charge_mode.map(Self::charge_mode_to_str),
            events: self.events.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        self.clone().set_all()
    }
}

impl OnPowerEvent for Battery {
    fn on_power_event(&mut self, new_mode: PowerMode) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        match new_mode {
            PowerMode::PluggedIn => {
                // plug event resets battery settings
                self.events
                    .iter_mut()
                    .for_each(|ev| ev.is_triggered = false);
                self.set_charge_mode().map_err(|e| vec![e])
            }
            PowerMode::PluggedOut => {
                // plug event resets battery settings
                self.events
                    .iter_mut()
                    .for_each(|ev| ev.is_triggered = false);
                self.set_charge_mode().map_err(|e| vec![e])
            }
            PowerMode::BatteryCharge(_) => Ok(()),
        }
        .unwrap_or_else(|mut e| errors.append(&mut e));
        for ev in &mut self.events {
            ev.on_power_event(new_mode)
                .unwrap_or_else(|mut e| errors.append(&mut e));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        crate::api::BatteryLimits {
            charge_current: Some(RangeLimit {
                min: self.limits.charge_rate.min,
                max: self.limits.charge_rate.max,
            }),
            charge_current_step: 50,
            charge_modes: vec![
                "normal".to_owned(),
                "discharge".to_owned(),
                "idle".to_owned(),
            ],
            charge_limit: Some(RangeLimit {
                min: 10.0,
                max: 90.0,
            }),
            charge_limit_step: 1.0,
        }
    }

    fn json(&self) -> crate::persist::BatteryJson {
        self.clone().into()
    }

    fn charge_rate(&mut self, rate: Option<u64>) {
        self.charge_rate = rate;
    }

    fn get_charge_rate(&self) -> Option<u64> {
        self.charge_rate
    }

    fn charge_mode(&mut self, mode: Option<String>) {
        self.charge_mode = mode.map(|s| Self::str_to_charge_mode(&s)).flatten()
    }

    fn get_charge_mode(&self) -> Option<String> {
        self.charge_mode.map(Self::charge_mode_to_str)
    }

    fn read_charge_full(&self) -> Option<f64> {
        match Self::read_charge_full() {
            Ok(x) => Some(x),
            Err(e) => {
                log::warn!("read_charge_full err: {}", e.msg);
                None
            }
        }
    }

    fn read_charge_now(&self) -> Option<f64> {
        match Self::read_charge_now() {
            Ok(x) => Some(x),
            Err(e) => {
                log::warn!("read_charge_now err: {}", e.msg);
                None
            }
        }
    }

    fn read_charge_design(&self) -> Option<f64> {
        match Self::read_charge_design() {
            Ok(x) => Some(x),
            Err(e) => {
                log::warn!("read_charge_design err: {}", e.msg);
                None
            }
        }
    }

    fn read_current_now(&self) -> Option<f64> {
        match Self::read_current_now() {
            Ok(x) => Some(x as f64),
            Err(e) => {
                log::warn!("read_current_now err: {}", e.msg);
                None
            }
        }
    }

    fn charge_limit(&mut self, limit: Option<f64>) {
        // upper limit
        let index = self.find_limit_event();
        if let Some(index) = index {
            if let Some(limit) = limit {
                log::info!(
                    "Updating Steam Deck charge limit event instruction to >{}",
                    limit
                );
                self.events[index] = EventInstruction {
                    trigger: EventTrigger::BatteryAbove(limit / 100.0),
                    charge_rate: None,
                    charge_mode: Some(ChargeMode::Idle),
                    is_triggered: false,
                };
            } else {
                self.events.remove(index);
            }
        } else if let Some(limit) = limit {
            log::info!(
                "Creating Steam Deck charge limit event instruction of >{}",
                limit
            );
            self.events.push(EventInstruction {
                trigger: EventTrigger::BatteryAbove(limit / 100.0),
                charge_rate: None,
                charge_mode: Some(ChargeMode::Idle),
                is_triggered: false,
            });
        }
        // lower limit
        let index = self.find_unlimit_event();
        if let Some(index) = index {
            if let Some(limit) = limit {
                let limit = (limit - 10.0).clamp(0.0, 100.0);
                log::info!(
                    "Updating Steam Deck charge limit event instruction to <{}",
                    limit
                );
                self.events[index] = EventInstruction {
                    trigger: EventTrigger::BatteryBelow(limit / 100.0),
                    charge_rate: None,
                    charge_mode: Some(ChargeMode::Normal),
                    is_triggered: false,
                };
            } else {
                self.events.remove(index);
            }
        } else if let Some(limit) = limit {
            let limit = (limit - 10.0).clamp(0.0, 100.0);
            log::info!(
                "Creating Steam Deck charge limit event instruction of <{}",
                limit
            );
            self.events.push(EventInstruction {
                trigger: EventTrigger::BatteryBelow(limit / 100.0),
                charge_rate: None,
                charge_mode: Some(ChargeMode::Normal),
                is_triggered: false,
            });
        }
    }

    fn get_charge_limit(&self) -> Option<f64> {
        let index = self.find_limit_event();
        if let Some(index) = index {
            if let EventTrigger::BatteryAbove(limit) = self.events[index].trigger {
                Some(limit * 100.0)
            } else {
                log::error!("Got index {} for battery charge limit which does not have expected event trigger: {:?}", index, &self.events);
                None
            }
        } else {
            None
        }
    }

    fn check_power(&mut self) -> Result<Vec<PowerMode>, Vec<SettingError>> {
        log::debug!("Steam Deck power vibe check");
        let mut errors = Vec::new();
        let mut events = Vec::new();
        match (Self::read_charge_full(), Self::read_charge_now()) {
            (Ok(full), Ok(now)) => events.push(PowerMode::BatteryCharge(now / full)),
            (Err(e1), Err(e2)) => {
                errors.push(e1);
                errors.push(e2);
            }
            (Err(e), _) => errors.push(e),
            (_, Err(e)) => errors.push(e),
        }
        match Self::read_usb_voltage() {
            Ok(voltage) => {
                if voltage > 0.0
                    && self.state.charger_state != crate::state::steam_deck::ChargeState::PluggedIn
                {
                    events.push(PowerMode::PluggedIn);
                    self.state.charger_state = crate::state::steam_deck::ChargeState::PluggedIn;
                } else if voltage == 0.0
                    && self.state.charger_state != crate::state::steam_deck::ChargeState::Unplugged
                {
                    events.push(PowerMode::PluggedOut);
                    self.state.charger_state = crate::state::steam_deck::ChargeState::Unplugged;
                }
            }
            Err(e) => errors.push(e),
        }
        if errors.is_empty() {
            Ok(events)
        } else {
            Err(errors)
        }
    }

    fn provider(&self) -> crate::persist::DriverJson {
        self.driver_mode.clone()
    }
}
