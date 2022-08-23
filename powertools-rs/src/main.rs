mod api;
mod persist;
mod settings;
mod state;

mod resume_worker;
mod save_worker;
mod utility;

use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::core::serdes::Primitive;
use usdpl_back::Instance;

const PORT: u16 = 44443;

const PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

const DEFAULT_SETTINGS_FILE: &str = "default_settings.json";

fn main() -> Result<(), ()> {
    let log_filepath = format!("/tmp/{}.log", PACKAGE_NAME);
    WriteLogger::init(
        #[cfg(debug_assertions)]
        {
            LevelFilter::Debug
        },
        #[cfg(not(debug_assertions))]
        {
            LevelFilter::Info
        },
        Default::default(),
        std::fs::File::create(&log_filepath).unwrap(),
    )
    .unwrap();
    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);

    let default_settings = persist::SettingsJson::open(settings_dir().join(DEFAULT_SETTINGS_FILE))
        .map(|settings| settings::Settings::from_json(settings, DEFAULT_SETTINGS_FILE.into()))
        .unwrap_or_else(|_| settings::Settings::system_default(DEFAULT_SETTINGS_FILE.into()));

    log::debug!("Settings: {:?}", default_settings);

    let (_save_handle, save_sender) = save_worker::spawn(default_settings.clone());
    let _resume_handle = resume_worker::spawn(default_settings.clone());

    Instance::new(PORT)
        .register("hello", |_: Vec<Primitive>| {
            vec![format!("Hello {}", PACKAGE_NAME).into()]
        })
        // battery API functions
        .register("BATTERY_current_now", api::battery::current_now)
        .register(
            "BATTERY_set_charge_rate",
            api::battery::set_charge_rate(default_settings.battery.clone(), save_sender.clone()),
        )
        .register(
            "BATTERY_get_charge_rate",
            api::battery::get_charge_rate(default_settings.battery.clone()),
        )
        .register(
            "BATTERY_unset_charge_rate",
            api::battery::unset_charge_rate(default_settings.battery.clone(), save_sender.clone()),
        )
        // cpu API functions
        .register("CPU_count", api::cpu::max_cpus)
        .register(
            "CPU_set_online",
            api::cpu::set_cpu_online(default_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_get_onlines",
            api::cpu::get_cpus_online(default_settings.cpus.clone())
        )
        .register(
            "CPU_set_clock_limits",
            api::cpu::set_clock_limits(default_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_get_clock_limits",
            api::cpu::get_clock_limits(default_settings.cpus.clone())
        )
        .register(
            "CPU_unset_clock_limits",
            api::cpu::unset_clock_limits(default_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_set_governor",
            api::cpu::set_cpu_governor(default_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_get_governors",
            api::cpu::get_cpu_governors(default_settings.cpus.clone())
        )
        // gpu API functions
        .register(
            "GPU_set_ppt",
            api::gpu::set_ppt(default_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_get_ppt",
            api::gpu::get_ppt(default_settings.gpu.clone())
        )
        .register(
            "GPU_unset_ppt",
            api::gpu::unset_ppt(default_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_set_clock_limits",
            api::gpu::set_clock_limits(default_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_get_clock_limits",
            api::gpu::get_clock_limits(default_settings.gpu.clone())
        )
        .register(
            "GPU_unset_clock_limits",
            api::gpu::unset_clock_limits(default_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_set_slow_memory",
            api::gpu::set_slow_memory(default_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_get_slow_memory",
            api::gpu::get_slow_memory(default_settings.gpu.clone())
        )
        .run_blocking()
}

fn settings_dir() -> std::path::PathBuf {
    usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/home/deck".into())
        .join(".config/powertools/")
}
