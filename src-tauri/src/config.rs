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
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const GLOBAL_CONFIG_FILE: &str = "global_config.json";

pub fn get_config_path(app_handle: &AppHandle) -> PathBuf {
    let mut path = app_handle
        .path()
        .app_config_dir()
        .expect("Could not get app config dir");
    path.push(GLOBAL_CONFIG_FILE);
    path
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranscendiaConfig {
    pub region: Option<Region>,
    pub monitor: u32,
    pub text_color: String,
    pub text_shadow_color: String,
    pub lang: String,
    pub show_fps: bool,
    pub resolution_multiplier: f32,
}

impl Default for TranscendiaConfig {
    fn default() -> Self {
        Self {
            region: None,
            monitor: 0,
            text_color: "#FFFFFF".to_string(),
            text_shadow_color: "#000000".to_string(),
            lang: "en".to_string(),
            show_fps: false,
            resolution_multiplier: 1.0,
        }
    }
}

impl TranscendiaConfig {
    pub fn load(app: &AppHandle) -> Self {
        let config_path = get_config_path(app);

        let config = if !config_path.exists() {
            let parent = config_path.parent().unwrap();
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Could not create config dir");
            }

            let data = serde_json::to_string(&TranscendiaConfig::default())
                .expect("Could not serialize default config");
            fs::write(&config_path, &data).expect("Could not write config.json");
            data
        } else {
            fs::read_to_string(&config_path).expect("Could not read config.json")
        };
        let config = serde_json::from_str::<TranscendiaConfig>(&config).unwrap_or_else(|_| {
            fs::remove_file(config_path).expect("Could not reset the config.json");
            Self::load(app)
        });
        config
    }

    #[inline(always)]
    pub fn save(&self, app: &AppHandle) {
        let config = serde_json::to_string(self).expect("Could not stringify config");
        fs::write(get_config_path(app), &config).expect("Could not write config.json");
    }
}

pub struct ConfigState(pub Mutex<TranscendiaConfig>);
