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
use objc2_app_kit::{NSPopUpMenuWindowLevel, NSView, NSWindowCollectionBehavior};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Must be called on the main thread.
pub fn make_popup_window(window: &winit::window::Window) {
    let Ok(handle) = window.window_handle() else {
        error!("No window handle");
        return;
    };
    let RawWindowHandle::AppKit(h) = handle.as_raw() else {
        error!("Not an AppKit window handle");
        return;
    };
    let view = h.ns_view.as_ptr() as *const NSView;
    if view.is_null() {
        error!("Invalid view pointer");
        return;
    }
    unsafe {
        let Some(ns_win) = (*view).window() else {
            error!("View has no window");
            return;
        };
        ns_win.setLevel(NSPopUpMenuWindowLevel);
        ns_win.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
    }
}

pub fn request_screen_record_permissions() {
    use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};
    if !CGPreflightScreenCaptureAccess() {
        error!("No permission for screen capture !");
        CGRequestScreenCaptureAccess();
    }
}
