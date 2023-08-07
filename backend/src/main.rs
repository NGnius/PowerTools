mod api;
mod persist;
mod settings;
mod state;

mod consts;
use consts::*;
mod power_worker;
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
    let log_filepath = usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(PACKAGE_NAME.to_owned() + ".log");
    #[cfg(not(debug_assertions))]
    let log_filepath = std::path::Path::new("/tmp").join(format!("{}.log", PACKAGE_NAME));
    #[cfg(debug_assertions)]
    let old_log_filepath = usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(PACKAGE_NAME.to_owned() + ".log.old");
    #[cfg(debug_assertions)]
    {
        if std::path::Path::new(&log_filepath).exists() {
            std::fs::copy(&log_filepath, &old_log_filepath)
                .expect("Unable to increment logs. Do you have write permissions?");
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
        //std::fs::File::create("/home/deck/powertools-rs.log").unwrap(),
    )
    .unwrap();
    log::debug!("Logging to: {:?}.", log_filepath);
    println!("Logging to: {:?}", log_filepath);
    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    log::info!(
        "Current dir `{}`",
        std::env::current_dir().unwrap().display()
    );
    println!(
        "Current dir `{}`",
        std::env::current_dir().unwrap().display()
    );

    log::info!("home dir: {:?}", usdpl_back::api::dirs::home());

    log::info!("Last version file: {}", utility::read_version_file());
    if let Err(e) = utility::save_version_file() {
        log::error!("Error storing version: {}", e);
    } else {
        log::info!("Updated version file succesfully");
    }

    let _limits_handle = crate::settings::limits_worker_spawn();

    let mut loaded_settings =
        persist::SettingsJson::open(utility::settings_dir().join(DEFAULT_SETTINGS_FILE))
            .map(|settings| settings::Settings::from_json(settings, DEFAULT_SETTINGS_FILE.into()))
            .unwrap_or_else(|_| {
                settings::Settings::system_default(
                    DEFAULT_SETTINGS_FILE.into(),
                    DEFAULT_SETTINGS_NAME.into(),
                )
            });

    log::info!(
        "Detected device automatically {:?}, using driver: {:?} (This can be overriden)",
        crate::settings::auto_detect_provider(), loaded_settings.cpus.provider()
    );

    log::debug!("Settings: {:?}", loaded_settings);

    let (api_handler, api_sender) = crate::api::handler::ApiMessageHandler::new();

    //let (_save_handle, save_sender) = save_worker::spawn(loaded_settings.clone());
    let _resume_handle = resume_worker::spawn(api_sender.clone());
    let _power_handle = power_worker::spawn(api_sender.clone());

    let (message_getter, message_dismisser) = api::message::MessageHandler::new().to_callables();

    let instance = Instance::new(PORT)
        .register("V_INFO", |_: Vec<Primitive>| {
            #[cfg(debug_assertions)]
            {
                vec![format!("v{}-dbg", PACKAGE_VERSION).into()]
            }
            #[cfg(not(debug_assertions))]
            {
                vec![format!("v{}-rls", PACKAGE_VERSION).into()]
            }
        })
        .register("NAME", |_: Vec<Primitive>| vec![PACKAGE_NAME.into()])
        .register("LOG", api::general::log_it())
        // battery API functions
        .register_async(
            "BATTERY_current_now",
            api::battery::current_now(api_sender.clone()),
        )
        .register_async(
            "BATTERY_charge_now",
            api::battery::charge_now(api_sender.clone()),
        )
        .register_async(
            "BATTERY_charge_full",
            api::battery::charge_full(api_sender.clone()),
        )
        .register_async(
            "BATTERY_charge_design",
            api::battery::charge_design(api_sender.clone()),
        )
        .register_async(
            "BATTERY_charge_power",
            api::battery::charge_power(api_sender.clone()),
        )
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
        .register(
            "BATTERY_set_charge_mode",
            api::battery::set_charge_mode(api_sender.clone()),
        )
        .register(
            "BATTERY_get_charge_mode",
            api::battery::get_charge_mode(api_sender.clone()),
        )
        .register(
            "BATTERY_unset_charge_mode",
            api::battery::unset_charge_mode(api_sender.clone()),
        )
        .register(
            "BATTERY_set_charge_limit",
            api::battery::set_charge_limit(api_sender.clone()),
        )
        .register(
            "BATTERY_unset_charge_limit",
            api::battery::unset_charge_limit(api_sender.clone()),
        )
        .register_async(
            "BATTERY_get_charge_limit",
            api::battery::get_charge_limit(api_sender.clone()),
        )
        // cpu API functions
        .register("CPU_count", api::cpu::max_cpus)
        .register(
            "CPU_set_online",
            api::cpu::set_cpu_online(api_sender.clone()),
        )
        .register(
            "CPU_set_onlines",
            api::cpu::set_cpus_online(api_sender.clone()),
        )
        .register_async(
            "CPU_get_onlines",
            api::cpu::get_cpus_online(api_sender.clone()),
        )
        .register_async("CPU_set_smt", api::cpu::set_smt(api_sender.clone()))
        .register_async("CPU_get_smt", api::cpu::get_smt(api_sender.clone()))
        .register(
            "CPU_set_clock_limits",
            api::cpu::set_clock_limits(api_sender.clone()),
        )
        .register(
            "CPU_get_clock_limits",
            api::cpu::get_clock_limits(api_sender.clone()),
        )
        .register(
            "CPU_unset_clock_limits",
            api::cpu::unset_clock_limits(api_sender.clone()),
        )
        .register(
            "CPU_set_governor",
            api::cpu::set_cpu_governor(api_sender.clone()),
        )
        .register(
            "CPU_set_governors",
            api::cpu::set_cpus_governors(api_sender.clone()),
        )
        .register(
            "CPU_get_governors",
            api::cpu::get_cpu_governors(api_sender.clone()),
        )
        // gpu API functions
        .register("GPU_set_ppt", api::gpu::set_ppt(api_sender.clone()))
        .register_async("GPU_get_ppt", api::gpu::get_ppt(api_sender.clone()))
        .register("GPU_unset_ppt", api::gpu::unset_ppt(api_sender.clone()))
        .register(
            "GPU_set_clock_limits",
            api::gpu::set_clock_limits(api_sender.clone()),
        )
        .register_async(
            "GPU_get_clock_limits",
            api::gpu::get_clock_limits(api_sender.clone()),
        )
        .register(
            "GPU_unset_clock_limits",
            api::gpu::unset_clock_limits(api_sender.clone()),
        )
        .register(
            "GPU_set_slow_memory",
            api::gpu::set_slow_memory(api_sender.clone()),
        )
        .register_async(
            "GPU_get_slow_memory",
            api::gpu::get_slow_memory(api_sender.clone()),
        )
        // general API functions
        .register(
            "GENERAL_set_persistent",
            api::general::set_persistent(api_sender.clone()),
        )
        .register(
            "GENERAL_get_persistent",
            api::general::get_persistent(api_sender.clone()),
        )
        .register(
            "GENERAL_load_settings",
            api::general::load_settings(api_sender.clone()),
        )
        .register(
            "GENERAL_load_default_settings",
            api::general::load_default_settings(api_sender.clone()),
        )
        .register(
            "GENERAL_load_system_settings",
            api::general::load_system_settings(api_sender.clone()),
        )
        .register_async(
            "GENERAL_get_name",
            api::general::get_name(api_sender.clone()),
        )
        .register_async(
            "GENERAL_get_path",
            api::general::get_path(api_sender.clone()),
        )
        .register_async(
            "GENERAL_wait_for_unlocks",
            api::general::lock_unlock_all(api_sender.clone()),
        )
        .register_blocking(
            "GENERAL_get_limits",
            api::general::get_limits(api_sender.clone()),
        )
        .register_async(
            "GENERAL_get_provider",
            api::general::get_provider(api_sender.clone()),
        )
        .register("GENERAL_idk", api::general::gunter)
        .register(
            "GENERAL_apply_now",
            api::general::force_apply(api_sender.clone()),
        )
        .register(
            "GENERAL_on_pluggedin",
            api::battery::on_plugged(api_sender.clone()),
        )
        .register(
            "GENERAL_on_unplugged",
            api::battery::on_unplugged(api_sender.clone()),
        )
        .register_async(
            "GENERAL_get_periodicals",
            api::general::get_periodicals(api_sender.clone())
        )
        .register_async("MESSAGE_get", message_getter)
        .register_async("MESSAGE_dismiss", message_dismisser);

    if let Err(e) = loaded_settings.on_set() {
        e.iter()
            .for_each(|e| log::error!("Startup Settings.on_set() error: {}", e));
    } else {
        log::info!("Startup Settings.on_set() success");
    }

    if let Err(e) = utility::chown_settings_dir() {
        log::warn!("Failed to change config dir permissions: {}", e);
    }

    api_worker::spawn(loaded_settings, api_handler);

    instance.run_blocking()
}
