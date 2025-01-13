use crate::config::{Config, ConfigState};
use crate::errors::TranscendiaError;
use crate::events::Events;
use crate::translate_runtime::{start_translate_runtime, TranslateRuntime};
use crate::windows::{
    create_config_window, create_overlay_window, create_select_region_window, edit_overlay,
};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

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
    runtime: tauri::State<'_, TranslateRuntime>,
    new_config: Config,
    refresh_w_overlay: bool,
) -> Result<(), TranscendiaError> {
    new_config.save(&app_handle);

    let mut config = config
        .0
        .lock()
        .map_err(|_| TranscendiaError::CannotSaveConfig)?;
    *config = new_config;

    runtime.interval.store(config.interval, Ordering::SeqCst);

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
pub async fn select_region(app_handle: AppHandle, monitor: i8) -> Result<(), tauri::Error> {
    create_select_region_window(&app_handle, monitor)?;

    let windows = app_handle.webview_windows();
    let window = windows.values().find(|x| x.label() == "overlay");
    if let Some(w) = window {
        w.close()?;
    }

    Ok(())
}

#[tauri::command]
pub async fn finish_select_region(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranslateRuntime>,
) -> Result<(), tauri::Error> {
    f_s_r(app_handle, config, runtime)
}

pub fn f_s_r(
    app_handle: AppHandle,
    config: tauri::State<'_, ConfigState>,
    runtime: tauri::State<'_, TranslateRuntime>,
) -> Result<(), tauri::Error> {
    let config = config.0.lock().expect("Cannot read config");
    if let Some(region) = &config.region {
        start_translate_runtime(&app_handle, &runtime);
        create_overlay_window(&app_handle, region, config.monitor, config.blur_background)?;
    }

    create_config_window(&app_handle)?;

    Ok(())
}
