// Copyright © 2025 Nantsa Montillet
// SPDX-License-Identifier: AGPL-3.0-or-later

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    transcendia_lib::run()
}
