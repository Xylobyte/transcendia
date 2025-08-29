// Copyright © 2025 Nantsa Montillet
// SPDX-License-Identifier: AGPL-3.0-or-later

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
