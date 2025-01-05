use crate::config::Region;
use crate::translate_runtime::start_translate_runtime;
use tauri::utils::config::WindowEffectsConfig;
use tauri::utils::{WindowEffect, WindowEffectState};
use tauri::webview::Color;
use tauri::{AppHandle, LogicalPosition, LogicalSize, WebviewWindow, WebviewWindowBuilder};

pub fn create_select_region_window(
    app: &AppHandle,
    monitor: i8,
) -> Result<WebviewWindow, tauri::Error> {
    let window =
        WebviewWindowBuilder::new(app, "select", tauri::WebviewUrl::App("select.html".into()))
            .title("Transcendia - Select a region")
            .accept_first_mouse(true)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .shadow(false)
            .background_color(Color(0, 0, 0, 0))
            .decorations(false)
            .disable_drag_drop_handler()
            .transparent(true)
            .resizable(false)
            .visible(false)
            .build()?;

    let monitors = window.available_monitors()?;
    if monitors.len() > monitor as usize {
        let monitor = monitors.get(monitor as usize).unwrap();
        let scale = monitor.scale_factor();
        window.set_position(monitor.position().to_logical::<f64>(scale))?;
        window.set_size(monitor.size().to_logical::<f64>(scale))?;
    }
    window.show()?;
    window.set_focus()?;

    Ok(window)
}

pub fn create_config_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let window =
        WebviewWindowBuilder::new(app, "config", tauri::WebviewUrl::App("config.html".into()))
            .title("Transcendia - Configuration")
            .accept_first_mouse(true)
            .always_on_top(true)
            .inner_size(400f64, 650f64)
            .resizable(false)
            .build()?;
    window.set_focus()?;

    Ok(window)
}

pub fn create_overlay_window(
    app: &AppHandle,
    region: &Region,
    monitor: u8,
    blur: bool,
    interval: u8,
) -> Result<WebviewWindow, tauri::Error> {
    let window = WebviewWindowBuilder::new(
        app,
        "overlay",
        tauri::WebviewUrl::App("overlay.html".into()),
    )
        .title("Transcendia - Overlay")
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .shadow(false)
        .background_color(Color(0, 0, 0, 0))
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .visible(false)
        .build()?;
    window.set_ignore_cursor_events(true)?;

    edit_overlay(&window, &region, monitor, blur)?;
    window.show()?;

    start_translate_runtime(&app, interval);

    Ok(window)
}

pub fn edit_overlay(
    window: &WebviewWindow,
    region: &Region,
    monitor: u8,
    blur: bool,
) -> Result<(), tauri::Error> {
    let monitors = window.available_monitors()?;
    if monitors.len() > monitor as usize {
        let monitor = monitors.get(monitor as usize).unwrap();
        let logical_position = monitor.position().to_logical::<i32>(monitor.scale_factor());
        window.set_position(LogicalPosition {
            x: logical_position.x + region.x,
            y: logical_position.y + region.y,
        })?;
        window.set_size(LogicalSize {
            width: region.w,
            height: region.h,
        })?;
    }

    if blur {
        window.set_effects(WindowEffectsConfig {
            effects: vec![WindowEffect::HudWindow],
            state: Some(WindowEffectState::Active),
            radius: Some(30f64),
            color: None,
        })?;
    }

    Ok(())
}
