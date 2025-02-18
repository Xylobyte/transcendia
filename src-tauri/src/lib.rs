mod commands;
mod config;
mod errors;
mod events;
mod systray;
mod translate_runtime;
mod windows;
mod ocr_models;

use crate::commands::{download_finish, f_s_r, finish_select_region, get_config, select_region, set_config};
use crate::config::{Config, ConfigState};
use crate::ocr_models::check_for_models;
use crate::systray::create_systray;
use crate::translate_runtime::{start_translate_runtime, TranslateRuntime};
use crate::windows::{create_config_window, create_overlay_window};
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::{Arc, Mutex};
use tauri::{generate_context, generate_handler, ActivationPolicy, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tokio::sync::Notify;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let close_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyX);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, shortcut, event| {
                    if shortcut == &close_shortcut && event.state == ShortcutState::Released {
                        let windows = _app.webview_windows();
                        let window = windows.values().find(|x| x.label() == "select");
                        if let Some(w) = window {
                            let state = _app.state::<ConfigState>();
                            let runtime = _app.state::<TranslateRuntime>();
                            f_s_r(_app.clone(), state, runtime, true).expect("Failed reopen windows");
                            w.close().expect("Failed to close window");
                        }
                    }
                })
                .with_shortcut(close_shortcut)
                .expect("Shortcut error")
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            create_systray(app)?;

            let app = app.handle();

            let config = Config::load(&app);
            app.manage(ConfigState(Mutex::new(config.clone())));

            let runtime = TranslateRuntime {
                need_stop: Arc::new(Notify::default()),
                is_running: Arc::new(AtomicBool::from(false)),
                interval: Arc::new(AtomicU8::new(config.interval)),
            };

            if check_for_models(&app) {
                if let Some(region) = config.region {
                    start_translate_runtime(&app, &runtime, config.monitor.clone(), region.clone());
                    create_overlay_window(&app, &region, &config.monitor, config.blur_background)?;
                } else {
                    create_config_window(&app)?;
                }
            }

            app.manage(runtime);

            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            select_region,
            finish_select_region,
            download_finish
        ])
        .run(generate_context!())
        .expect("Error while running Transcendia");
}
