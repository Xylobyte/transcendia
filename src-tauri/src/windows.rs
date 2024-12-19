use crate::config::Region;
use tauri::webview::Color;
use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder};

pub fn create_select_region_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
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
            .maximized(true)
            .resizable(false)
            .build()?;

    Ok(window)
}

pub fn create_config_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let window =
        WebviewWindowBuilder::new(app, "config", tauri::WebviewUrl::App("config.html".into()))
            .title("Transcendia - Configuration")
            .always_on_top(true)
            .inner_size(400f64, 600f64)
            .resizable(false)
            .build()?;

    Ok(window)
}

pub fn create_overlay_window(
    app: &AppHandle,
    region: &Region,
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
        .background_color(Color(0, 0, 0, 1))
        .decorations(false)
        //.transparent(true)
        .resizable(false)
        .position(region.x, region.y)
        .inner_size(region.w, region.h)
        .build()?;
    window.set_ignore_cursor_events(true)?;

    Ok(window)
}
