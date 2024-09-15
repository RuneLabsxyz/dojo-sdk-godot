use godot::prelude::*;
use log::LevelFilter;
use crate::logging::GodotLogSink;

pub mod macros;
pub mod world;
pub mod client;

pub mod logging;

struct DojoSdkExtension;

static LOGGER: GodotLogSink = GodotLogSink {};


#[gdextension]
unsafe impl ExtensionLibrary for DojoSdkExtension {
    fn on_level_init(level: InitLevel) {
        #[cfg(debug_assertions)]
        let log_level = LevelFilter::Debug;
        #[cfg(not(debug_assertions))]
        let log_level = LevelFilter::Info;

        if level == InitLevel::Scene {
            // Register logging
            if let Err(e) = log::set_logger(&LOGGER)
                .map(|()| log::set_max_level(log_level)) {
                godot_error!("Welp, we cannot log. An error occurred: {}", e)
            }
        }
    }
}