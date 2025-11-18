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
use crate::config::{ConfigState, TranscendiaConfig};
use crate::errors::Result;
use crate::errors::TranscendiaError;
use crate::events::Events;
use crate::monitors::{BaseTranscendiaMonitor, TranscendiaMonitor};
use crate::runtime::runtime::TranscendiaRuntime;
use crate::windows::{create_config_window, create_select_region_window, move_overlay};
use tauri::{AppHandle, Emitter, Manager};
use xcap::Monitor;

#[tauri::command]
pub fn get_config(config: tauri::State<'_, ConfigState>) -> Result<TranscendiaConfig> {
    let config = config
        .0
        .lock()
        .map_err(|_| TranscendiaError::CannotLoadConfig)?;
    Ok(config.clone())
}

#[tauri::command]
pub async fn set_config(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranscendiaRuntime>,
    new_config: TranscendiaConfig,
    reload_runtime: bool,
    monitor_changed: bool,
) -> Result<()> {
    new_config.save(&app_handle);

    let mut config = config
        .0
        .lock()
        .map_err(|_| TranscendiaError::CannotSaveConfig)?;
    *config = new_config;

    app_handle
        .emit(Events::RefreshOverlay.as_str(), None::<bool>)
        .expect("Failed to emit event");

    if monitor_changed {
        let window = app_handle.get_webview_window("overlay");
        if let Some(w) = window {
            move_overlay(&w, config.monitor).expect("Failed to move overlay");
        }
    }

    if reload_runtime {
        runtime.update_config(config.clone());
    }

    Ok(())
}

#[tauri::command]
pub fn get_monitors() -> Result<Vec<BaseTranscendiaMonitor>> {
    Monitor::get_all()
}

#[tauri::command]
pub async fn select_region(app_handle: AppHandle, monitor: u32) -> Result<()> {
    create_select_region_window(&app_handle, monitor)
        .map_err(|_| TranscendiaError::WindowOpenError)?;

    Ok(())
}

#[tauri::command]
pub async fn finish_select_region(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranscendiaRuntime>,
) -> Result<()> {
    f_s_r(app_handle, config, runtime)
}

pub fn f_s_r(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranscendiaRuntime>,
) -> Result<()> {
    let config = config.0.lock().expect("Cannot read config");
    runtime.update_config(config.clone());

    create_config_window(&app_handle).map_err(|_| TranscendiaError::WindowOpenError)?;

    Ok(())
}
