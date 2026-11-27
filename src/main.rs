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

mod app;
mod config;
mod errors;
pub mod events;
mod gfx;
mod hotkeys;
mod models;
mod monitors;
mod paths;
mod platform_specifics;
mod runtime;
mod tray;
mod ui;

use app::App;
use events::UserEvent;
use global_hotkey::GlobalHotKeyEvent;
use single_instance::SingleInstance;
use std::sync::Arc;
use tray_icon::menu::MenuEvent;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let guard = SingleInstance::new("com.xylobyte.transcendia")
        .expect("Cannot create single instance guard");
    if !guard.is_single() {
        log::warn!("Transcendia is already running");
        return;
    }

    let mut event_loop = EventLoop::<UserEvent>::with_user_event();
    platform_specifics::macos::with_activation_policy(&mut event_loop);
    let event_loop = event_loop.build().expect("Cannot create event loop");

    let proxy = event_loop.create_proxy();

    let p = proxy.clone();
    MenuEvent::set_event_handler(Some(Arc::new(move |e| {
        let _ = p.send_event(UserEvent::Menu(e));
    })));
    let p = proxy.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |e| {
        let _ = p.send_event(UserEvent::Hotkey(e));
    }));

    let mut app = App::new(proxy);
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Event loop error: {e}");
    }
}
