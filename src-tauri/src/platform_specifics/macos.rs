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
use objc2_app_kit::{NSPopUpMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
use tauri::WebviewWindow;

#[inline(always)]
pub fn make_popup_window(window: &WebviewWindow) -> Result<(), tauri::Error> {
    #[cfg(target_os = "macos")]
    unsafe {
        let ns_win = window.ns_window()? as *mut NSWindow;
        (*ns_win).setLevel(NSPopUpMenuWindowLevel);
        (*ns_win).setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
    }

    Ok(())
}