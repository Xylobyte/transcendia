use crate::config::{Config, ConfigState};
use crate::errors::TranscendiaError;
use crate::windows::create_select_region_window;
use tauri::AppHandle;

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
    config: tauri::State<'_, ConfigState>,
    app_handle: AppHandle,
    new_config: Config,
) -> Result<(), TranscendiaError> {
    new_config.save(&app_handle);

    let mut config = config
        .0
        .lock()
        .map_err(|_| TranscendiaError::CannotSaveConfig)?;
    *config = new_config;

    Ok(())
}

#[tauri::command]
pub async fn select_region(app_handle: AppHandle) -> Result<(), tauri::Error> {
    create_select_region_window(&app_handle)?;
    Ok(())
}
