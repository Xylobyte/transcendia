/*
 * Copyright © 2025 Nantsa Montillet
 * SPDX-License-Identifier: AGPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod commands;
mod config;
mod errors;
mod events;
mod runtime;
mod systray;
mod windows;
pub mod monitors;

use crate::commands::{
    download_finish, f_s_r, finish_select_region, get_config, get_monitors, select_region,
    set_config,
};
use crate::config::{Config, ConfigState};
use crate::runtime::ocr::TranscendiaOcr;
use crate::systray::create_systray;
use crate::windows::{create_config_window, create_overlay_window};
use runtime::runtime::TranscendiaRuntime;
use std::sync::Mutex;
use tauri::{generate_context, generate_handler, ActivationPolicy, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let mut builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let close_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyX);

        builder = builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, shortcut, event| {
                    if shortcut == &close_shortcut && event.state == ShortcutState::Released {
                        let windows = _app.webview_windows();
                        let window = windows.values().find(|x| x.label() == "select");
                        if let Some(w) = window {
                            let state = _app.state::<ConfigState>();
                            let runtime = _app.state::<TranscendiaRuntime>();
                            f_s_r(_app.clone(), state, runtime, true)
                                .expect("Failed reopen windows");
                            w.close().expect("Failed to close window");
                        }
                    }
                })
                .with_shortcut(close_shortcut)
                .expect("Shortcut error")
                .build(),
        );
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_macos_permissions::init())
    }

    builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            create_systray(app)?;

            let app = app.handle();

            let config = Config::load(app);
            app.manage(ConfigState(Mutex::new(config.clone())));

            let runtime = TranscendiaRuntime::new(config.interval);

            if TranscendiaOcr::check(app) {
                if let Some(region) = config.region {
                    runtime.start(
                        app,
                        config.monitor.clone(),
                        region.clone(),
                        config.lang.clone(),
                    );
                    create_overlay_window(app, &region, config.monitor, config.blur_background)?;
                } else {
                    create_config_window(app)?;
                }
            }

            app.manage(runtime);

            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            get_monitors,
            select_region,
            finish_select_region,
            download_finish
        ])
        .run(generate_context!())
        .expect("Error while running Transcendia");
}
