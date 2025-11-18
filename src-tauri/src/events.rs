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
use crate::config::ConfigState;
use crate::runtime::runtime::TranscendiaRuntime;
use crate::windows::create_overlay_window;
use log::debug;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};

pub enum Events {
    OnOffConfigTrayItem,
    ToggleOverlay,
    RefreshOverlay,
    NewTranslatedText,
}

impl Events {
    pub fn as_str(&self) -> &'static str {
        match self {
            Events::OnOffConfigTrayItem => "OnOffConfigTrayItem",
            Events::RefreshOverlay => "RefreshOverlay",
            Events::NewTranslatedText => "NewTranslatedText",
            Events::ToggleOverlay => "ToggleOverlay",
        }
    }
}

pub fn handle_run_event(app_handle: &AppHandle, event: RunEvent) {
    match event {
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
        RunEvent::WindowEvent { label, event, .. } if label == "select" => match event {
            WindowEvent::Destroyed => {
                let config = app_handle.state::<ConfigState>();
                create_overlay_window(app_handle, config.0.lock().unwrap().monitor).unwrap();
            }
            WindowEvent::Focused { .. } => {
                let window = app_handle.get_webview_window("overlay");
                if let Some(w) = window {
                    w.close().unwrap();
                }
            }
            _ => {}
        },
        RunEvent::ExitRequested { api, code, .. } => {
            debug!("Received exit request with code {:?}", code);
            if code.is_none() {
                api.prevent_exit();
            }
        }
        _ => {}
    }
}
