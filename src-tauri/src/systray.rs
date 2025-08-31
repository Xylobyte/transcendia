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

use crate::events::Events;
use crate::windows::create_config_window;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{App, Listener};

pub fn create_systray(app: &App) -> Result<TrayIcon, tauri::Error> {
    let config_item_on = MenuItem::with_id(app, "config", "Configuration", true, None::<&str>)?;
    let config_item_off = MenuItem::with_id(app, "config", "Configuration", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Transcendia", true, None::<&str>)?;
    let menu_on = Menu::with_items(app, &[&config_item_on, &quit_item])?;
    let menu_off = Menu::with_items(app, &[&config_item_off, &quit_item])?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu_on)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "config" => {
                if let Err(err) = create_config_window(app) {
                    println!("Failed to create config window : {:?}", err);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {
                println!("Menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;

    let tray_clone = tray.clone();
    app.listen(Events::OnOffConfigTrayItem.as_str(), move |event| {
        if event.payload() == "true" {
            tray_clone.set_menu(Some(menu_on.clone())).unwrap();
        } else {
            tray_clone.set_menu(Some(menu_off.clone())).unwrap();
        }
    });

    Ok(tray)
}
