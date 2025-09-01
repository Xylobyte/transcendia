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
use crate::runtime::ocr::{DETECTION_MODEL_NAME, MODEL_FOLDER_NAME, RECOGNITION_MODEL_NAME};
use log::{debug, error};
use reqwest::{Client, Url};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_macos_permissions::{
    check_screen_recording_permission, request_screen_recording_permission,
};
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};
use xcap::Monitor;

pub struct TranscendiaRuntime {
    need_stop: Arc<Notify>,
    is_running: Arc<AtomicBool>,
    interval: Arc<AtomicU8>,
}

impl TranscendiaRuntime {
    pub fn new(interval: u8) -> Self {
        Self {
            need_stop: Arc::new(Notify::default()),
            is_running: Arc::new(AtomicBool::new(false)),
            interval: Arc::new(AtomicU8::new(interval)),
        }
    }

    pub fn update(&self, interval: u8) {
        self.interval.store(interval, Ordering::Relaxed);
    }

    pub fn start(&self, app_handle: &AppHandle, monitor: u32, region: Region, lang: String) {
        if self.is_running.load(Ordering::Relaxed) {
            return;
        }

        let app_handle = app_handle.clone();
        let interval = self.interval.clone();
        let need_stop = self.need_stop.clone();

        tauri::async_runtime::spawn(async move {
            #[cfg(target_os = "macos")]
            if !check_screen_recording_permission().await {
                error!("No permission for screen capture !");
                request_screen_recording_permission().await;
                return;
            }

            let monitor = Monitor::load(monitor);

            let models_folder = app_handle
                .path()
                .app_config_dir()
                .expect("Could not get app config dir")
                .join(MODEL_FOLDER_NAME);

            // Load ocr engine here

            let client = Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(20))
                .https_only(true)
                .build()
                .expect("Could not create HTTP client");

            let mut old_text = String::new();

            loop {
                tokio::select! {
                    _ = need_stop.notified() => {
                        break;
                    }
                    _ = sleep(Duration::from_secs(interval.load(Ordering::Relaxed) as u64)) => {
                        let start = Instant::now();

                        app_handle.emit(Events::NewTranslatedText.as_str(), "".to_string()).unwrap();

                        debug!("Time to translate the screen: {}", start.elapsed().as_millis());
                    }
                }
            }
        });

        self.is_running.store(true, Ordering::Release);
    }

    pub fn stop(&self) {
        self.need_stop.notify_one();
        self.is_running.store(false, Ordering::Release);
    }
}

async fn translate_text(text: &mut String, target_lang: &str, client: &Client) {
    let original_linebreaks = ["\r\n", "\n", "\r"];
    let mut processed_text = text.clone();
    for lb in &original_linebreaks {
        processed_text = processed_text.replace(lb, "\u{200B}");
    }

    let mut url = Url::parse(&format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t",
        target_lang
    ))
        .unwrap();
    url.query_pairs_mut().append_pair("q", &processed_text);

    let res = client.get(url).send().await;

    if let Ok(r) = res {
        let res_text = r.text().await.expect("Could not read response");
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
