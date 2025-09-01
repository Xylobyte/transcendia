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

use crate::config::{Config, ConfigState};
use crate::errors::TranscendiaError;
use crate::events::Events;
use crate::monitors::{BaseTranscendiaMonitor, TranscendiaMonitor};
use crate::runtime::runtime::TranscendiaRuntime;
use crate::windows::{
    create_config_window, create_overlay_window, create_select_region_window, edit_overlay,
};
use tauri::{AppHandle, Emitter, Manager};
use xcap::Monitor;

#[tauri::command]
pub fn get_config(config: tauri::State<'_, ConfigState>) -> Result<Config, TranscendiaError> {
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
    new_config: Config,
    refresh_w_overlay: bool,
) -> Result<(), TranscendiaError> {
    new_config.save(&app_handle);

    let mut config = config
        .0
        .lock()
        .map_err(|_| TranscendiaError::CannotSaveConfig)?;
    *config = new_config;

    runtime.update(config.interval);

    if refresh_w_overlay {
        let windows = app_handle.webview_windows();
        let window = windows.values().find(|x| x.label() == "overlay");
        if let (Some(w), Some(r)) = (window, config.region.clone()) {
            edit_overlay(w, &r, config.monitor, config.blur_background)
                .expect("Failed to edit overlay");
        }
    } else {
        app_handle
            .emit(Events::RefreshOverlay.as_str(), None::<bool>)
            .expect("Failed to emit event");
    }

    Ok(())
}

#[tauri::command]
pub fn get_monitors() -> Result<Vec<BaseTranscendiaMonitor>, TranscendiaError> {
    Monitor::get_all()
}

#[tauri::command]
pub async fn select_region(
    app_handle: AppHandle,
    runtime: tauri::State<'_, TranscendiaRuntime>,
    monitor: u32,
) -> Result<(), tauri::Error> {
    create_select_region_window(&app_handle, monitor)?;

    let windows = app_handle.webview_windows();
    let window = windows.values().find(|x| x.label() == "overlay");
    if let Some(w) = window {
        w.close()?;
        runtime.stop();
    }

    Ok(())
}

#[tauri::command]
pub async fn finish_select_region(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranscendiaRuntime>,
) -> Result<(), tauri::Error> {
    f_s_r(app_handle, config, runtime, true)
}

#[tauri::command]
pub async fn download_finish(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranscendiaRuntime>,
) -> Result<(), tauri::Error> {
    f_s_r(app_handle, config, runtime, false)
}

pub fn f_s_r(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranscendiaRuntime>,
    create_config: bool,
) -> Result<(), tauri::Error> {
    let config = config.0.lock().expect("Cannot read config");
    let region = &config.region;
    if let Some(r) = region {
        runtime.start(
            &app_handle,
            config.monitor.clone(),
            r.clone(),
            config.lang.clone(),
        );
        create_overlay_window(&app_handle, r, config.monitor, config.blur_background)?;
    }

    if create_config || region.is_none() {
        create_config_window(&app_handle)?;
    }

    Ok(())
}
