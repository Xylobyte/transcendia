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
use crate::monitors::TranscendiaMonitor;
use log::debug;
use tauri::webview::Color;
use tauri::Manager;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tauri_nspanel::{tauri_panel, PanelLevel, WebviewWindowExt};
use xcap::Monitor;

tauri_panel! {
    panel!(SelectPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: true,
            is_floating_panel: true,
            works_when_modal: true,
        }
    })

    panel!(OverlayPanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
            is_floating_panel: true,
            works_when_modal: true,
            hides_on_deactivate: false,
        }
    })

    panel_event!(BasePanelEventHandler {
        window_did_become_key(notification: &NSNotification) -> (),
        window_did_resign_key(notification: &NSNotification) -> (),
        window_will_close(notification: &NSNotification) -> (),
        window_did_miniaturize(notification: &NSNotification) -> (),
        window_did_deminiaturize(notification: &NSNotification) -> ()
    })
}

pub fn create_select_region_window(app: &AppHandle, monitor: u32) -> Result<(), tauri::Error> {
    let window = WebviewWindowBuilder::new(app, "select", WebviewUrl::App("select.html".into()))
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

    let monitor = Monitor::load(monitor);
    let scale = monitor.scale_factor().unwrap();
    window.set_position(LogicalPosition {
        x: monitor.x().unwrap() as f32 * scale,
        y: monitor.y().unwrap() as f32 * scale,
    })?;
    window.set_size(LogicalSize {
        width: monitor.width().unwrap() as f32,
        height: monitor.height().unwrap() as f32,
    })?;

    #[cfg(target_os = "macos")]
    {
        let panel = window.to_panel::<SelectPanel>()?;
        panel.set_level(PanelLevel::PopUpMenu.value());
        panel.show_and_make_key();
    }
    #[cfg(not(target_os = "macos"))]
    {
        window.show()?;
        window.set_focus()?;
    }

    Ok(())
}

pub fn create_config_window(app: &AppHandle) -> Result<(), tauri::Error> {
    let window = WebviewWindowBuilder::new(app, "config", WebviewUrl::App("config.html".into()))
        .title("Transcendia - Configuration")
        .accept_first_mouse(true)
        .always_on_top(true)
        .inner_size(400f64, 650f64)
        .resizable(false)
        .build()?;
    window.set_focus()?;

    Ok(())
}

pub fn create_overlay_window(app: &AppHandle, monitor: u32) -> Result<(), tauri::Error> {
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
    move_overlay(&window, monitor)?;

    #[cfg(target_os = "macos")]
    {
        let handler = BasePanelEventHandler::new();
        handler.window_did_become_key(|notification| {
            debug!("Handling key event: {:?}", notification);
        });
        handler.window_did_resign_key(|notification| {
            debug!("Handling key resign event: {:?}", notification);
        });
        handler.window_will_close(|notification| {
            debug!("Handling close event: {:?}", notification);
        });
        handler.window_did_miniaturize(|notification| {
            debug!("Handling mini event: {:?}", notification);
        });

        let panel = window.to_panel::<OverlayPanel>()?;
        panel.set_level(PanelLevel::PopUpMenu.value());
        panel.set_event_handler(Some(handler.as_ref()));
        panel.show();
    }
    #[cfg(not(target_os = "macos"))]
    window.show()?;

    Ok(())
}

pub fn move_overlay(window: &WebviewWindow, monitor: u32) -> Result<(), tauri::Error> {
    let monitor = Monitor::load(monitor);
    let sf = monitor.scale_factor().unwrap();
    let pos = LogicalPosition {
        x: monitor.x().unwrap() as f32 * sf,
        y: monitor.y().unwrap() as f32 * sf,
    };
    window.set_position(pos)?;
    window.set_size(LogicalSize {
        width: monitor.width().unwrap(),
        height: monitor.height().unwrap(),
    })?;

    Ok(())
}
