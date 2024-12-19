mod commands;
mod config;
mod errors;
mod events;
mod systray;
mod windows;

use crate::commands::{get_config, select_region, set_config};
use crate::config::{Config, ConfigState};
use crate::systray::create_systray;
use crate::windows::{create_config_window, create_overlay_window};
use std::sync::Mutex;
use tauri::{generate_context, generate_handler, ActivationPolicy, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            _app.set_activation_policy(ActivationPolicy::Accessory);

            create_systray(_app)?;

            let app = _app.handle();

            let config = Config::load(&app);
            app.manage(ConfigState(Mutex::new(config.clone())));

            if let Some(region) = config.region {
                create_overlay_window(&app, &region)?;
            } else {
                create_config_window(&app)?;
            }

            Ok(())
        })
        .invoke_handler(generate_handler![get_config, set_config, select_region])
        .run(generate_context!())
        .expect("Error while running Transcendia");
}
