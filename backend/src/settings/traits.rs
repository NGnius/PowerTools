use std::fmt::Debug;
use super::SettingError;
use super::MinMax;

pub trait OnSet {
    fn on_set(&mut self) -> Result<(), SettingError>;
}

pub trait OnResume {
    fn on_resume(&self) -> Result<(), SettingError>;
}

pub trait SettingsRange {
    fn max() -> Self;
    fn min() -> Self;
}

pub trait TGpu: OnResume + OnSet + Debug + Send {
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

pub trait TCpus: OnResume + OnSet + Debug + Send {
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

pub trait TGeneral: OnResume + OnSet + Debug + Send {
    fn limits(&self) -> crate::api::GeneralLimits;

    fn get_persistent(&self) -> bool;

    fn persistent(&mut self) -> &'_ mut bool;

    fn get_path(&self) -> &'_ std::path::Path;

    fn path(&mut self, path: std::path::PathBuf);

    fn get_name(&self) -> &'_ str;

    fn name(&mut self, name: String);

    fn provider(&self) -> crate::persist::DriverJson;
}

pub trait TBattery: OnResume + OnSet + Debug + Send {
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

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::AutoDetect
    }
}
