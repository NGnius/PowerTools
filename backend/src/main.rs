mod api;
mod persist;
mod settings;
mod state;

mod consts;
use consts::*;
mod resume_worker;
mod save_worker;
mod utility;

use settings::OnSet;

use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::core::serdes::Primitive;
use usdpl_back::Instance;

fn main() -> Result<(), ()> {
    let log_filepath = format!("/home/deck/{}.log", PACKAGE_NAME);
    #[cfg(debug_assertions)]
    {
        if std::path::Path::new(&log_filepath).exists() {
            std::fs::copy(&log_filepath, "/home/deck/powertools.log.old").unwrap();
        }
    }
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

    let mut loaded_settings = persist::SettingsJson::open(utility::settings_dir().join(DEFAULT_SETTINGS_FILE))
        .map(|settings| settings::Settings::from_json(settings, DEFAULT_SETTINGS_FILE.into()))
        .unwrap_or_else(|_| settings::Settings::system_default(DEFAULT_SETTINGS_FILE.into()));

    log::debug!("Settings: {:?}", loaded_settings);

    let (_save_handle, save_sender) = save_worker::spawn(loaded_settings.clone());
    let _resume_handle = resume_worker::spawn(loaded_settings.clone());

    if let Err(e) = loaded_settings.on_set() {
        log::error!("Startup Settings.on_set() error: {}", e);
    }

    Instance::new(PORT)
        .register("V_INFO", |_: Vec<Primitive>| {
            vec![format!("{} v{}", PACKAGE_NAME, PACKAGE_VERSION).into()]
        })
        // battery API functions
        .register("BATTERY_current_now", api::battery::current_now)
        .register(
            "BATTERY_set_charge_rate",
            api::battery::set_charge_rate(loaded_settings.battery.clone(), save_sender.clone()),
        )
        .register(
            "BATTERY_get_charge_rate",
            api::battery::get_charge_rate(loaded_settings.battery.clone()),
        )
        .register(
            "BATTERY_unset_charge_rate",
            api::battery::unset_charge_rate(loaded_settings.battery.clone(), save_sender.clone()),
        )
        // cpu API functions
        .register("CPU_count", api::cpu::max_cpus)
        .register(
            "CPU_set_online",
            api::cpu::set_cpu_online(loaded_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_get_onlines",
            api::cpu::get_cpus_online(loaded_settings.cpus.clone())
        )
        .register(
            "CPU_set_clock_limits",
            api::cpu::set_clock_limits(loaded_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_get_clock_limits",
            api::cpu::get_clock_limits(loaded_settings.cpus.clone())
        )
        .register(
            "CPU_unset_clock_limits",
            api::cpu::unset_clock_limits(loaded_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_set_governor",
            api::cpu::set_cpu_governor(loaded_settings.cpus.clone(), save_sender.clone())
        )
        .register(
            "CPU_get_governors",
            api::cpu::get_cpu_governors(loaded_settings.cpus.clone())
        )
        // gpu API functions
        .register(
            "GPU_set_ppt",
            api::gpu::set_ppt(loaded_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_get_ppt",
            api::gpu::get_ppt(loaded_settings.gpu.clone())
        )
        .register(
            "GPU_unset_ppt",
            api::gpu::unset_ppt(loaded_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_set_clock_limits",
            api::gpu::set_clock_limits(loaded_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_get_clock_limits",
            api::gpu::get_clock_limits(loaded_settings.gpu.clone())
        )
        .register(
            "GPU_unset_clock_limits",
            api::gpu::unset_clock_limits(loaded_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_set_slow_memory",
            api::gpu::set_slow_memory(loaded_settings.gpu.clone(), save_sender.clone())
        )
        .register(
            "GPU_get_slow_memory",
            api::gpu::get_slow_memory(loaded_settings.gpu.clone())
        )
        // general API functions
        .register(
            "GENERAL_set_persistent",
            api::general::set_persistent(loaded_settings.general.clone(), save_sender.clone())
        )
        .register(
            "GENERAL_get_persistent",
            api::general::get_persistent(loaded_settings.general.clone())
        )
        .register(
            "GENERAL_load_settings",
            api::general::load_settings(loaded_settings.clone())
        )
        .register(
            "GENERAL_load_default_settings",
            api::general::load_default_settings(loaded_settings.clone())
        )
        .register(
            "GENERAL_get_name",
            api::general::get_name(loaded_settings.general.clone())
        )
        .register(
            "GENERAL_wait_for_unlocks",
            api::general::lock_unlock_all(loaded_settings.clone())
        )
        .run_blocking()
}
