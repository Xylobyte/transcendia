// Copyright © 2025 Nantsa Montillet
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum TranscendiaError {
    CannotLoadConfig,
    CannotSaveConfig,
    CannotGetMonitors,
}
