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
use crate::events::Events;
use crate::windows::{create_config_window, create_overlay_window};
use log::{debug, error, warn};
use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{App, Listener, Manager};

pub fn create_systray(app: &App) -> Result<TrayIcon, tauri::Error> {
    let info_item = MenuItem::with_id(
        app,
        "info",
        format!("Transcendia - v{}", app.config().clone().version.unwrap()),
        false,
        None::<&str>,
    )?;
    let config_item = MenuItem::with_id(app, "config", "Configuration", true, None::<&str>)?;
    let show_overlay = CheckMenuItem::with_id(
        app,
        "show_overlay",
        "Show overlay",
        true,
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Transcendia", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&info_item)
        .separator()
        .items(&[&config_item, &show_overlay])
        .separator()
        .item(&quit_item)
        .build()?;

    let show_overlay_clone = show_overlay.clone();
    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "config" => {
                if let Err(err) = create_config_window(app) {
                    error!("Failed to create config window : {:?}", err);
                }
            }
            "quit" => {
                app.exit(0);
            }
            "show_overlay" => {
                let window = app.get_webview_window("overlay");
                if let Some(w) = window {
                    w.close().unwrap();
                    show_overlay_clone.set_checked(false).unwrap();
                } else {
                    let config = app.state::<ConfigState>();
                    create_overlay_window(&app, config.0.lock().unwrap().monitor).unwrap();
                    show_overlay_clone.set_checked(true).unwrap();
                }
            }
            _ => {
                warn!("Menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;

    let config_item_clone = config_item.clone();
    app.listen(Events::OnOffConfigTrayItem.as_str(), move |event| {
        config_item_clone
            .set_enabled(event.payload() == "true")
            .expect("Failed to set config item enabled state");
    });

    Ok(tray)
}
