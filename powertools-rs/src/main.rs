mod persist;
mod settings;

use simplelog::{WriteLogger, LevelFilter};

use usdpl_back::Instance;
use usdpl_back::core::serdes::Primitive;

const PORT: u16 = 44443;

const PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), ()> {
    let log_filepath = format!("/tmp/{}.log", PACKAGE_NAME);
    WriteLogger::init(
        #[cfg(debug_assertions)]{LevelFilter::Debug},
        #[cfg(not(debug_assertions))]{LevelFilter::Info},
        Default::default(),
        std::fs::File::create(&log_filepath).unwrap()
    ).unwrap();
    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    
    let default_settings: settings::Settings = persist::SettingsJson::open(
        settings_dir().join("default_settings.json")
    ).unwrap_or_default().into();
    
    log::debug!("Settings: {:?}", default_settings);
    
    Instance::new(PORT)
        .register("hello", |_: Vec<Primitive>| vec![format!("Hello {}", PACKAGE_NAME).into()])
        .run_blocking()
}

fn settings_dir() -> std::path::PathBuf {
    usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/home/deck".into())
        .join(".config/powertools/")
}
