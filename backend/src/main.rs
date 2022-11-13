mod api;
mod persist;
mod settings;
mod state;

mod consts;
use consts::*;
mod resume_worker;
//mod save_worker;
mod api_worker;
mod utility;

use settings::OnSet;

use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::core::serdes::Primitive;
use usdpl_back::Instance;

fn main() -> Result<(), ()> {
    #[cfg(debug_assertions)]
    let log_filepath = format!("/home/deck/{}.log", PACKAGE_NAME);
    #[cfg(not(debug_assertions))]
    let log_filepath = format!("/tmp/{}.log", PACKAGE_NAME);
    #[cfg(debug_assertions)]
    {
        if std::path::Path::new(&log_filepath).exists() {
            std::fs::copy(&log_filepath, format!("/home/deck/{}.log.old", PACKAGE_NAME)).unwrap();
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

    let (api_handler, api_sender) = crate::api::handler::ApiMessageHandler::new();

    //let (_save_handle, save_sender) = save_worker::spawn(loaded_settings.clone());
    let _resume_handle = resume_worker::spawn(api_sender.clone());

    if let Err(e) = loaded_settings.on_set() {
        log::error!("Startup Settings.on_set() error: {}", e);
    }

    let instance = Instance::new(PORT)
        .register("V_INFO", |_: Vec<Primitive>| {
            vec![format!("{} v{}", PACKAGE_NAME, PACKAGE_VERSION).into()]
        })
        // battery API functions
        .register("BATTERY_current_now", api::battery::current_now)
        .register("BATTERY_charge_now", api::battery::charge_now)
        .register("BATTERY_charge_full", api::battery::charge_full)
        .register("BATTERY_charge_design", api::battery::charge_design)
        .register(
            "BATTERY_set_charge_rate",
            api::battery::set_charge_rate(api_sender.clone()),
        )
        .register(
            "BATTERY_get_charge_rate",
            api::battery::get_charge_rate(api_sender.clone()),
        )
        .register(
            "BATTERY_unset_charge_rate",
            api::battery::unset_charge_rate(api_sender.clone()),
        )
        // cpu API functions
        .register("CPU_count", api::cpu::max_cpus)
        .register(
            "CPU_set_online",
            api::cpu::set_cpu_online(api_sender.clone())
        )
        .register(
            "CPU_set_onlines",
            api::cpu::set_cpus_online(api_sender.clone())
        )
        .register_async(
            "CPU_get_onlines",
            api::cpu::get_cpus_online(api_sender.clone())
        )
        .register_async(
            "CPU_set_smt",
            api::cpu::set_smt(api_sender.clone())
        )
        .register(
            "CPU_set_clock_limits",
            api::cpu::set_clock_limits(api_sender.clone())
        )
        .register(
            "CPU_get_clock_limits",
            api::cpu::get_clock_limits(api_sender.clone())
        )
        .register(
            "CPU_unset_clock_limits",
            api::cpu::unset_clock_limits(api_sender.clone())
        )
        .register(
            "CPU_set_governor",
            api::cpu::set_cpu_governor(api_sender.clone())
        )
        .register(
            "CPU_set_governors",
            api::cpu::set_cpus_governors(api_sender.clone())
        )
        .register(
            "CPU_get_governors",
            api::cpu::get_cpu_governors(api_sender.clone())
        )
        // gpu API functions
        .register(
            "GPU_set_ppt",
            api::gpu::set_ppt(api_sender.clone())
        )
        .register_async(
            "GPU_get_ppt",
            api::gpu::get_ppt(api_sender.clone())
        )
        .register(
            "GPU_unset_ppt",
            api::gpu::unset_ppt(api_sender.clone())
        )
        .register(
            "GPU_set_clock_limits",
            api::gpu::set_clock_limits(api_sender.clone())
        )
        .register_async(
            "GPU_get_clock_limits",
            api::gpu::get_clock_limits(api_sender.clone())
        )
        .register(
            "GPU_unset_clock_limits",
            api::gpu::unset_clock_limits(api_sender.clone())
        )
        .register(
            "GPU_set_slow_memory",
            api::gpu::set_slow_memory(api_sender.clone())
        )
        .register_async(
            "GPU_get_slow_memory",
            api::gpu::get_slow_memory(api_sender.clone())
        )
        // general API functions
        .register(
            "GENERAL_set_persistent",
            api::general::set_persistent(api_sender.clone())
        )
        .register(
            "GENERAL_get_persistent",
            api::general::get_persistent(api_sender.clone())
        )
        .register(
            "GENERAL_load_settings",
            api::general::load_settings(api_sender.clone())
        )
        .register(
            "GENERAL_load_default_settings",
            api::general::load_default_settings(api_sender.clone())
        )
        .register(
            "GENERAL_load_system_settings",
            api::general::load_system_settings(api_sender.clone())
        )
        .register_async(
            "GENERAL_get_name",
            api::general::get_name(api_sender.clone())
        )
        .register_async(
            "GENERAL_wait_for_unlocks",
            api::general::lock_unlock_all(api_sender.clone())
        );

    api_worker::spawn(loaded_settings, api_handler);

    instance
        .run_blocking()
}
