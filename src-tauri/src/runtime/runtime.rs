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

use crate::config::Region;
use crate::events::Events;
use crate::monitors::TranscendiaMonitor;
use crate::runtime::ocr::TranscendiaOcr;
use log::{debug, error};
use reqwest::blocking::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tauri_plugin_macos_permissions::{
    check_screen_recording_permission, request_screen_recording_permission,
};
use tokio::time::Duration;
use xcap::Monitor;

pub struct TranscendiaRuntime {
    is_running: Arc<AtomicBool>,
}

impl TranscendiaRuntime {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, app_handle: &AppHandle, monitor: u32, region: Region, lang: String) {
        if self.is_running.load(Ordering::Relaxed) {
            return;
        }

        let is_running = self.is_running.clone();
        let app_handle = app_handle.clone();

        tauri::async_runtime::spawn_blocking(move || {
            #[cfg(target_os = "macos")]
            if !tauri::async_runtime::block_on(check_screen_recording_permission()) {
                error!("No permission for screen capture !");
                tauri::async_runtime::block_on(request_screen_recording_permission());
                return;
            }

            let monitor = Monitor::load(monitor);
            let mut ocr_engine = TranscendiaOcr::new();
            let client = Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(20))
                .https_only(true)
                .build()
                .expect("Could not create HTTP client");

            loop {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }

                let start = Instant::now();

                let image = monitor.capture_and_crop(&region);
                let texts = ocr_engine.extract(image);

                app_handle
                    .emit(Events::NewTranslatedText.as_str(), texts)
                    .unwrap();

                debug!(
                    "Time to translate the screen: {}ms",
                    start.elapsed().as_millis()
                );
            }
        });

        self.is_running.store(true, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Release);
    }
}

/*fn translate_text(text: &mut String, target_lang: &str, client: &Client) {
    let original_linebreaks = ["\r\n", "\n", "\r"];
    let mut processed_text = text.clone();
    for lb in &original_linebreaks {
        processed_text = processed_text.replace(lb, "\u{200B}");
    }

    let mut url = Url::parse(&format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t", // Other option : https://github.com/ssut/py-googletrans/issues/268
        target_lang
    ))
        .unwrap();
    url.query_pairs_mut().append_pair("q", &processed_text);

    let response = client.get(url).send();

    if let Ok(r) = response {
        let res_text = r.text().expect("Could not read response");
        let json = serde_json::from_str::<Value>(&res_text).expect("Could not parse json");
        if let Some(values) = json.get(0).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|i| i.get(0).and_then(|t| t.as_str()))
                .collect::<Vec<&str>>()
        }) {
            text.clear();
            for value in values {
                let restored = value.replace("\u{200B}", "\n");
                text.push_str(&restored);
            }
        } else {
            error!("Could not find translated text in response");
        }
    }
}
*/