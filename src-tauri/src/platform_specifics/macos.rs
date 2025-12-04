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
use log::error;
use objc2_app_kit::{NSPopUpMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
use tauri::{AppHandle, WebviewWindow};
use tauri_plugin_macos_permissions::{
    check_screen_recording_permission, request_screen_recording_permission,
};

#[inline(always)]
pub fn make_popup_window(handle: &AppHandle, window: WebviewWindow) -> Result<(), tauri::Error> {
    #[cfg(target_os = "macos")]
    handle.run_on_main_thread(move || unsafe {
        let ns_win_ptr = window.ns_window().unwrap() as *mut NSWindow;
        if ns_win_ptr.is_null() {
            error!("Invalid window pointer");
            return;
        }

        (*ns_win_ptr).setLevel(NSPopUpMenuWindowLevel);
        (*ns_win_ptr).setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
    })?;

    Ok(())
}

#[inline(always)]
pub fn request_screen_record_permissions() {
    #[cfg(target_os = "macos")]
    if !tauri::async_runtime::block_on(check_screen_recording_permission()) {
        error!("No permission for screen capture !");
        tauri::async_runtime::block_on(request_screen_recording_permission());
        return;
    }
}
