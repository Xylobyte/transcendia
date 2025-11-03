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
pub mod models;
pub mod monitors;
mod runtime;
mod systray;
mod windows;

use crate::commands::{
    f_s_r, finish_select_region, get_config, get_monitors, select_region, set_config,
};
use crate::config::{ConfigState, TranscendiaConfig};
use crate::events::Events;
use crate::systray::create_systray;
use crate::windows::create_overlay_window;
use log::debug;
use runtime::runtime::TranscendiaRuntime;
use std::sync::Mutex;
use tauri::{
    generate_context, generate_handler, ActivationPolicy, Emitter, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let mut builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let close_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyX);
        let toggle_overlay = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT);

        builder = builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state == ShortcutState::Released {
                        if shortcut == &close_shortcut {
                            let window = app.get_webview_window("select");
                            if let Some(w) = window {
                                let state = app.state::<ConfigState>();
                                let runtime = app.state::<TranscendiaRuntime>();
                                f_s_r(app.clone(), state, runtime).expect("Failed reopen windows");
                                w.close().expect("Failed to close window");
                            }
                        } else if shortcut == &toggle_overlay {
                            debug!("Shortcut not implemented");
                        }
                    }
                })
                .with_shortcuts([close_shortcut, toggle_overlay])
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

            let config = TranscendiaConfig::load(app);
            app.manage(ConfigState(Mutex::new(config.clone())));

            let runtime = TranscendiaRuntime::new(ConfigState(Mutex::new(config.clone())));
            runtime.start(app);
            app.manage(runtime);

            create_overlay_window(app, config.monitor)?;

            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            get_monitors,
            select_region,
            finish_select_region
        ])
        .build(generate_context!())
        .expect("Error while running Transcendia")
        .run(|app_handle, event| match event {
            RunEvent::WindowEvent { label, event, .. } if label == "config" => match event {
                WindowEvent::Destroyed { .. } => app_handle
                    .emit(Events::OnOffConfigTrayItem.as_str(), true)
                    .expect("Event error..."),
                WindowEvent::Focused { .. } => app_handle
                    .emit(Events::OnOffConfigTrayItem.as_str(), false)
                    .expect("Event error..."),
                _ => {}
            },
            RunEvent::WindowEvent { label, event, .. } if label == "overlay" => {
                let runtime = app_handle.state::<TranscendiaRuntime>();
                match event {
                    WindowEvent::Destroyed => {
                        runtime.stop();
                    }
                    WindowEvent::Focused { .. } => {
                        runtime.start(app_handle);
                    }
                    _ => {}
                }
            }
            RunEvent::WindowEvent { label, event, .. } if label == "select" => {
                match event {
                    WindowEvent::Destroyed => {
                        let config = app_handle.state::<ConfigState>();
                        create_overlay_window(app_handle, config.0.lock().unwrap().monitor)
                            .unwrap();
                    }
                    WindowEvent::Focused { .. } => {
                        let window = app_handle.get_webview_window("overlay");
                        if let Some(w) = window {
                            w.close().unwrap();
                        }
                    }
                    _ => {}
                }
            }
            RunEvent::ExitRequested { api, code, .. } => {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
            _ => {}
        });
}
