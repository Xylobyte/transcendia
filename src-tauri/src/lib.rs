mod windows;
mod config;

use crate::config::{Config, ConfigState};
use crate::windows::{create_config_window, create_overlay_window};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app = app.handle();

            let config = Config::load(app);
            app.manage(ConfigState(Mutex::new(config.clone())));

            if let Some(region) = config.region {
                create_overlay_window(app, &region)?;
            } else {
                create_config_window(app)?;
            }

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
