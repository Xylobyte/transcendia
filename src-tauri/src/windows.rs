// Copyright © 2025 Nantsa Montillet
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::config::Region;
use tauri::utils::config::WindowEffectsConfig;
use tauri::utils::{WindowEffect, WindowEffectState};
use tauri::webview::Color;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub fn create_select_region_window(
    app: &AppHandle,
    monitor: u32,
) -> Result<WebviewWindow, tauri::Error> {
    let window =
        WebviewWindowBuilder::new(app, "select", WebviewUrl::App("select.html".into()))
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

    let monitors = xcap::Monitor::all().expect("Could not retrieve monitors");
    let monitor = monitors
        .iter()
        .find(|m| m.id().unwrap() == monitor)
        .unwrap_or(monitors.get(0).expect("Cannot find any monitor"));
    let scale = monitor.scale_factor().unwrap();
    window.set_position(LogicalPosition { x: monitor.x().unwrap() as f32 * scale, y: monitor.y().unwrap() as f32 * scale })?;
    window.set_size(LogicalSize { width: monitor.width().unwrap() as f32, height: monitor.height().unwrap() as f32 })?;
    window.show()?;
    window.set_focus()?;

    Ok(window)
}

pub fn create_config_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let window = WebviewWindowBuilder::new(app, "config", WebviewUrl::App("config.html".into()))
        .title("Transcendia - Configuration")
        .accept_first_mouse(true)
        .always_on_top(true)
        .inner_size(400f64, 650f64)
        .resizable(false)
        .build()?;
    window.set_focus()?;

    Ok(window)
}

pub fn create_download_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let window =
        WebviewWindowBuilder::new(app, "downloader", WebviewUrl::App("models.html".into()))
            .title("Transcendia - Downloader")
            .always_on_top(true)
            .accept_first_mouse(true)
            .inner_size(500f64, 310f64)
            .resizable(false)
            .build()?;
    window.set_focus()?;

    Ok(window)
}

pub fn create_overlay_window(
    app: &AppHandle,
    region: &Region,
    monitor: u32,
    blur: bool,
) -> Result<WebviewWindow, tauri::Error> {
    let window = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("Transcendia - Overlay")
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .shadow(false)
        .background_color(Color(0, 0, 0, 0))
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .visible(false)
        .content_protected(true)
        .build()?;
    window.set_ignore_cursor_events(true)?;

    edit_overlay(&window, &region, monitor, blur)?;
    window.show()?;

    Ok(window)
}

pub fn edit_overlay(
    window: &WebviewWindow,
    region: &Region,
    monitor: u32,
    blur: bool,
) -> Result<(), tauri::Error> {
    let monitors = xcap::Monitor::all().expect("Could not retrieve monitors");
    let monitor = monitors
        .iter()
        .find(|m| m.id().unwrap() == monitor)
        .unwrap_or(monitors.get(0).expect("Cannot find any monitor"));
    window.set_position(LogicalPosition {
        x: monitor.x().unwrap() as f32 * monitor.scale_factor().unwrap() + region.x as f32,
        y: monitor.y().unwrap() as f32 * monitor.scale_factor().unwrap() + region.y as f32,
    })?;
    window.set_size(LogicalSize {
        width: region.w,
        height: region.h,
    })?;

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
