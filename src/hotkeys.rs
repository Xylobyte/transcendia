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
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;
use log::error;

pub struct Hotkeys {
    manager: GlobalHotKeyManager,
    pub toggle: HotKey,
    pub finish: HotKey,
    finish_registered: bool,
}

impl Hotkeys {
    /// Must be created on the main thread.
    pub fn new() -> Option<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| error!("Cannot create hotkey manager: {e}"))
            .ok()?;
        let toggle = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyT);
        let finish = HotKey::new(Some(Modifiers::CONTROL), Code::KeyX);
        if let Err(e) = manager.register(toggle) {
            error!("Cannot register Cmd+Shift+T: {e}");
        }

        Some(Self {
            manager,
            toggle,
            finish,
            finish_registered: false,
        })
    }

    pub fn register_finish(&mut self) {
        if !self.finish_registered {
            match self.manager.register(self.finish) {
                Ok(()) => self.finish_registered = true,
                Err(e) => error!("Cannot register Ctrl+X: {e}"),
            }
        }
    }

    pub fn unregister_finish(&mut self) {
        if self.finish_registered {
            let _ = self.manager.unregister(self.finish);
            self.finish_registered = false;
        }
    }
}
