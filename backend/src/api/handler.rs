use std::sync::mpsc::{self, Receiver, Sender};

use crate::settings::{Settings, TCpus, TGpu, TBattery, TGeneral, OnSet, OnResume, MinMax};
use crate::persist::SettingsJson;
use crate::utility::unwrap_maybe_fatal;

type Callback<T> = Box<dyn FnOnce(T) + Send>;

pub enum ApiMessage {
    Battery(BatteryMessage),
    Cpu(CpuMessage),
    Gpu(GpuMessage),
    General(GeneralMessage),
    OnResume,
    WaitForEmptyQueue(Callback<()>),
    LoadSettings(String, String), // (path, name)
    LoadMainSettings,
    LoadSystemSettings,
    GetLimits(Callback<super::SettingsLimits>),
    GetProvider(String, Callback<crate::persist::DriverJson>),
}

pub enum BatteryMessage {
    SetChargeRate(Option<u64>),
    GetChargeRate(Callback<Option<u64>>),
    SetChargeMode(Option<String>),
    GetChargeMode(Callback<Option<String>>),
    ReadChargeFull(Callback<Option<f64>>),
    ReadChargeNow(Callback<Option<f64>>),
    ReadChargeDesign(Callback<Option<f64>>),
    ReadCurrentNow(Callback<Option<f64>>),
}

impl BatteryMessage {
    fn process(self, settings: &mut dyn TBattery) -> bool {
        let dirty = self.is_modify();
        match self {
            Self::SetChargeRate(rate) => settings.charge_rate(rate),
            Self::GetChargeRate(cb) => cb(settings.get_charge_rate()),
            Self::SetChargeMode(mode) => settings.charge_mode(mode),
            Self::GetChargeMode(cb) => cb(settings.get_charge_mode()),
            Self::ReadChargeFull(cb) => cb(settings.read_charge_full()),
            Self::ReadChargeNow(cb) => cb(settings.read_charge_now()),
            Self::ReadChargeDesign(cb) => cb(settings.read_charge_design()),
            Self::ReadCurrentNow(cb) => cb(settings.read_current_now()),
        }
        dirty
    }

    /// Message instructs the driver to modify settings
    fn is_modify(&self) -> bool {
        matches!(self, Self::SetChargeRate(_) | Self::SetChargeMode(_))
    }
}

pub enum CpuMessage {
    SetCpuOnline(usize, bool),
    SetCpusOnline(Vec<bool>),
    SetSmt(bool, Callback<Vec<bool>>),
    GetSmt(Callback<bool>),
    GetCpusOnline(Callback<Vec<bool>>),
    SetClockLimits(usize, Option<MinMax<u64>>),
    GetClockLimits(usize, Callback<Option<MinMax<u64>>>),
    SetCpuGovernor(usize, String),
    SetCpusGovernor(Vec<String>),
    GetCpusGovernor(Callback<Vec<String>>),
}

impl CpuMessage {
    fn process(self, settings: &mut dyn TCpus) -> bool {
        let dirty = self.is_modify();
        // NOTE: "cpu" refers to the Linux kernel definition of a CPU, which is actually a hardware thread
        // not to be confused with a CPU chip, which usually has multiple hardware threads (cpu cores/threads) in the chip
        match self {
            Self::SetCpuOnline(index, status) => {settings.cpus().get_mut(index).map(|c| *c.online() = status);},
            Self::SetCpusOnline(cpus) => {
                for i in 0..cpus.len() {
                    settings.cpus().get_mut(i).map(|c| *c.online() = cpus[i]);
                }
            },
            Self::SetSmt(status, cb) => {
                if *settings.smt() == status {
                    // already set, do nothing
                } else if status {
                    // set SMT on
                    *settings.smt() = true;
                    let mut should_be_online = false;
                    let cpu_count = settings.len();
                    for i in (0..cpu_count).rev() {
                        if *settings.cpus()[i].online() && !should_be_online {
                            should_be_online = true;
                            // enable the odd-numbered thread right before
                            // for 1c:2t configs (i.e. anything with SMT2), the highest cpu core is always odd
                            // (e.g. 4c8t has CPUs 0-7, inclusive)
                            // this enables the """fake""" (i.e. odd) cpu which is disabled when SMT is set off
                            if i % 2 == 0 && i+1 != cpu_count {
                                *(settings.cpus()[i+1].online()) = true;
                            }
                        } else {
                            *settings.cpus()[i].online() = should_be_online;
                        }
                    }
                } else {
                    // set SMT off
                    *settings.smt() = false;
                    for i in 0..settings.len() {
                        // this disables the """fake""" (odd) cpu for appearances' sake
                        // the kernel will automatically disable that same cpu when SMT is changed
                        *settings.cpus()[i].online() = *settings.cpus()[i].online() && (status || i % 2 == 0);
                    }
                }
                let mut result = Vec::with_capacity(settings.len());
                for i in 0..settings.len() {
                    result.push(*settings.cpus()[i].online());
                }
                cb(result);
            },
            Self::GetSmt(cb) => {
                cb(*settings.smt());
            },
            Self::GetCpusOnline(cb) => {
                let mut result = Vec::with_capacity(settings.len());
                for cpu in settings.cpus() {
                    result.push(*cpu.online());
                }
                cb(result);
            },
            Self::SetClockLimits(index, clocks) => {settings.cpus().get_mut(index).map(|c| c.clock_limits(clocks));},
            Self::GetClockLimits(index, cb) => {settings.cpus().get(index).map(|c| cb(c.get_clock_limits().map(|x| x.to_owned())));},
            Self::SetCpuGovernor(index, gov) => {settings.cpus().get_mut(index).map(|c| c.governor(gov));},
            Self::SetCpusGovernor(govs) => {
                for i in 0..govs.len() {
                    settings.cpus().get_mut(i).map(|c| c.governor(govs[i].clone()));
                }
            },
            Self::GetCpusGovernor(cb) => {
                let mut result = Vec::with_capacity(settings.len());
                for cpu in settings.cpus() {
                    result.push(cpu.get_governor().to_owned());
                }
                cb(result);
            }
        }
        dirty
    }

    /// Message instructs the driver to modify settings
    fn is_modify(&self) -> bool {
        matches!(self,
            Self::SetCpuOnline(_, _)
            | Self::SetCpusOnline(_)
            | Self::SetSmt(_, _)
            | Self::SetClockLimits(_, _)
            | Self::SetCpuGovernor(_, _)
            | Self::SetCpusGovernor(_)
        )
    }
}

pub enum GpuMessage {
    SetPpt(Option<u64>, Option<u64>), // (fast, slow)
    GetPpt(Callback<(Option<u64>, Option<u64>)>),
    SetClockLimits(Option<MinMax<u64>>),
    GetClockLimits(Callback<Option<MinMax<u64>>>),
    SetSlowMemory(bool),
    GetSlowMemory(Callback<bool>),
}

impl GpuMessage {
    fn process(self, settings: &mut dyn TGpu) -> bool {
        let dirty = self.is_modify();
        match self {
            Self::SetPpt(fast, slow) => settings.ppt(fast, slow),
            Self::GetPpt(cb) => cb(settings.get_ppt()),
            Self::SetClockLimits(clocks) => settings.clock_limits(clocks),
            Self::GetClockLimits(cb) => cb(settings.get_clock_limits().map(|x| x.to_owned())),
            Self::SetSlowMemory(val) => *settings.slow_memory() = val,
            Self::GetSlowMemory(cb) => cb(*settings.slow_memory()),
        }
        dirty
    }

    fn is_modify(&self) -> bool {
        matches!(self,
            Self::SetPpt(_, _)
            | Self::SetClockLimits(_)
            | Self::SetSlowMemory(_)
        )
    }
}

pub enum GeneralMessage {
    SetPersistent(bool),
    GetPersistent(Callback<bool>),
    GetCurrentProfileName(Callback<String>),
}

impl GeneralMessage {
    fn process(self, settings: &mut dyn TGeneral) -> bool {
        let dirty = self.is_modify();
        match self {
            Self::SetPersistent(val) => *settings.persistent() = val,
            Self::GetPersistent(cb) => cb(*settings.persistent()),
            Self::GetCurrentProfileName(cb) => cb(settings.get_name().to_owned()),
        }
        dirty
    }

    fn is_modify(&self) -> bool {
        matches!(self, Self::SetPersistent(_))
    }
}

pub struct ApiMessageHandler {
    intake: Receiver<ApiMessage>,
    on_empty: Vec<Callback<()>>,
}

impl ApiMessageHandler {
    pub fn process_forever(&mut self, settings: &mut Settings) {
        while let Ok(msg) = self.intake.recv() {
            let mut dirty = self.process(settings, msg);
            while let Ok(msg) = self.intake.try_recv() {
                dirty |= self.process(settings, msg);
            }
            if dirty {
                // run on_set
                if let Err(e) = settings.on_set() {
                    log::error!("Settings on_set() err: {}", e);
                }
                // do callbacks
                for func in self.on_empty.drain(..) {
                    func(());
                }
                // save
                log::debug!("api_worker is saving...");
                let is_persistent = *settings.general.persistent();
                let save_path = crate::utility::settings_dir()
                    .join(settings.general.get_path().clone());
                if is_persistent {
                    let settings_clone = settings.json();
                    let save_json: SettingsJson = settings_clone.into();
                    unwrap_maybe_fatal(save_json.save(&save_path), "Failed to save settings");
                    log::debug!("Saved settings to {}", save_path.display());
                } else {
                    if save_path.exists() {
                        if let Err(e) = std::fs::remove_file(&save_path) {
                            log::warn!("Failed to delete persistent settings file {}: {}", save_path.display(), e);
                        } else {
                            log::debug!("Deleted persistent settings file {}", save_path.display());
                        }
                    } else {
                        log::debug!("Ignored save request for non-persistent settings");
                    }
                }
            } else {
                log::debug!("Skipping callbacks for non-modify handled message(s)");
            }
        }
    }

    pub fn process(&mut self, settings: &mut Settings, message: ApiMessage) -> bool {
        match message {
            ApiMessage::Battery(x) => x.process(settings.battery.as_mut()),
            ApiMessage::Cpu(x) => x.process(settings.cpus.as_mut()),
            ApiMessage::Gpu(x) => x.process(settings.gpu.as_mut()),
            ApiMessage::General(x) => x.process(settings.general.as_mut()),
            ApiMessage::OnResume => {
                if let Err(e) = settings.on_resume() {
                    log::error!("Settings on_resume() err: {}", e);
                }
                false
            }
            ApiMessage::WaitForEmptyQueue(callback) => {
                self.on_empty.push(callback);
                false
            },
            ApiMessage::LoadSettings(path, name) => {
                match settings.load_file(path.into(), name, false) {
                    Ok(success) => log::info!("Loaded settings file? {}", success),
                    Err(e) => log::warn!("Load file err: {}", e),
                }
                true
            }
            ApiMessage::LoadMainSettings => {
                match settings.load_file(
                    crate::consts::DEFAULT_SETTINGS_FILE.into(),
                    crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
                    true
                ) {
                    Ok(success) => log::info!("Loaded main settings file? {}", success),
                    Err(e) => log::warn!("Load file err: {}", e),
                }
                true
            }
            ApiMessage::LoadSystemSettings => {
                settings.load_system_default();
                true
            },
            ApiMessage::GetLimits(cb) => {
                cb(super::SettingsLimits {
                    battery: settings.battery.limits(),
                    cpu: settings.cpus.limits(),
                    gpu: settings.gpu.limits(),
                    general: settings.general.limits(),
                });
                false
            },
            ApiMessage::GetProvider(name, cb) => {
                cb(match &name as &str {
                    "battery" => settings.battery.provider(),
                    "cpu" | "cpus" => settings.cpus.provider(),
                    "gpu" => settings.gpu.provider(),
                    _ => settings.general.provider(),
                });
                false
            }
        }
    }

    pub fn new() -> (Self, Sender<ApiMessage>) {
        let (tx, rx) = mpsc::channel();
        (Self {
            intake: rx,
            on_empty: Vec::with_capacity(4),
        }, tx)
    }
}
