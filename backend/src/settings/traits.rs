use super::MinMax;
use super::SettingError;
use std::fmt::Debug;

pub trait OnSet {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>>;
}

pub trait OnResume {
    fn on_resume(&self) -> Result<(), Vec<SettingError>>;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum PowerMode {
    PluggedIn,
    PluggedOut,         // unplugged
    BatteryCharge(f64), // battery fill amount: 0 = empty, 1 = full
}

pub trait OnPowerEvent {
    fn on_plugged_in(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }

    fn on_plugged_out(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }

    fn on_charge_amount(&mut self, _amount: f64) -> Result<(), Vec<SettingError>> {
        Ok(())
    }

    fn on_power_event(&mut self, new_mode: PowerMode) -> Result<(), Vec<SettingError>> {
        match new_mode {
            PowerMode::PluggedIn => self.on_plugged_in(),
            PowerMode::PluggedOut => self.on_plugged_out(),
            PowerMode::BatteryCharge(now) => self.on_charge_amount(now),
        }
    }
}

pub trait TGpu: OnSet + OnResume + OnPowerEvent + Debug + Send {
    fn limits(&self) -> crate::api::GpuLimits;

    fn json(&self) -> crate::persist::GpuJson;

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>);

    fn get_ppt(&self) -> (Option<u64>, Option<u64>);

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>);

    fn get_clock_limits(&self) -> Option<&MinMax<u64>>;

    fn slow_memory(&mut self) -> &mut bool;

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::AutoDetect
    }
}

pub trait TCpus: OnSet + OnResume + OnPowerEvent + Debug + Send {
    fn limits(&self) -> crate::api::CpusLimits;

    fn json(&self) -> Vec<crate::persist::CpuJson>;

    fn cpus(&mut self) -> Vec<&mut dyn TCpu>;

    fn len(&self) -> usize;

    fn smt(&mut self) -> &'_ mut bool;

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::AutoDetect
    }
}

pub trait TCpu: Debug + Send {
    fn online(&mut self) -> &mut bool;

    fn governor(&mut self, governor: String);

    fn get_governor(&self) -> &'_ str;

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>);

    fn get_clock_limits(&self) -> Option<&MinMax<u64>>;
}

pub trait TGeneral: OnSet + OnResume + OnPowerEvent + Debug + Send {
    fn limits(&self) -> crate::api::GeneralLimits;

    fn get_persistent(&self) -> bool;

    fn persistent(&mut self) -> &'_ mut bool;

    fn get_path(&self) -> &'_ std::path::Path;

    fn path(&mut self, path: std::path::PathBuf);

    fn get_name(&self) -> &'_ str;

    fn name(&mut self, name: String);

    fn provider(&self) -> crate::persist::DriverJson;

    fn on_event(&self) -> &'_ crate::persist::OnEventJson;
}

pub trait TBattery: OnSet + OnResume + OnPowerEvent + Debug + Send {
    fn limits(&self) -> crate::api::BatteryLimits;

    fn json(&self) -> crate::persist::BatteryJson;

    fn charge_rate(&mut self, rate: Option<u64>);

    fn get_charge_rate(&self) -> Option<u64>;

    fn charge_mode(&mut self, mode: Option<String>);

    fn get_charge_mode(&self) -> Option<String>;

    fn read_charge_full(&self) -> Option<f64>;

    fn read_charge_now(&self) -> Option<f64>;

    fn read_charge_design(&self) -> Option<f64>;

    fn read_current_now(&self) -> Option<f64>;

    fn read_charge_power(&self) -> Option<f64>;

    fn charge_limit(&mut self, limit: Option<f64>);

    fn get_charge_limit(&self) -> Option<f64>;

    fn check_power(&mut self) -> Result<Vec<PowerMode>, Vec<SettingError>> {
        log::warn!("Power event check using default trait implementation");
        let mut events = Vec::new();
        if let (Some(full), Some(now)) = (self.read_charge_full(), self.read_charge_now()) {
            events.push(PowerMode::BatteryCharge(now / full));
        }
        Ok(events)
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::AutoDetect
    }
}
