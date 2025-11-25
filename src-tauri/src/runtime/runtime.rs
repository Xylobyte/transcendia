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
use crate::config::{ConfigState, TranscendiaConfig};
use crate::events::Events;
use crate::monitors::TranscendiaMonitor;
use crate::platform_specifics::macos;
use crate::runtime::ocr::TranscendiaOcr;
use log::debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use xcap::Monitor;

pub struct TranscendiaRuntime {
    need_stop: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    config: Arc<ConfigState>,
}

impl TranscendiaRuntime {
    #[inline]
    pub fn new(config: ConfigState) -> Self {
        Self {
            need_stop: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(config),
        }
    }

    pub fn start(&self, app_handle: &AppHandle) {
        if self.is_running.load(Ordering::Relaxed) {
            self.need_stop.store(false, Ordering::Release);
            return;
        }

        let need_stop = self.need_stop.clone();
        let is_running = self.is_running.clone();
        let config = self.config.clone();
        let app_handle = app_handle.clone();

        tauri::async_runtime::spawn_blocking(move || {
            macos::request_screen_record_permissions();

            let mut monitor_id = config.0.lock().unwrap().monitor;
            let mut monitor = Monitor::load(monitor_id);
            let mut ocr_engine = TranscendiaOcr::new(&app_handle);

            loop {
                if need_stop.load(Ordering::Relaxed) {
                    need_stop.store(false, Ordering::Release);
                    is_running.store(false, Ordering::Release);
                    break;
                }

                let m_lock = config.0.lock().unwrap();
                if m_lock.monitor != monitor_id {
                    monitor_id = m_lock.monitor;
                    monitor = Monitor::load(monitor_id);
                }
                let region = m_lock.region.clone();
                let resolution_multiplier = 1.4; // m_lock.scale_factor.clone();
                drop(m_lock);

                let mut time = Instant::now();

                let image = monitor.capture_and_crop(resolution_multiplier, &region);

                let capture_time = time.elapsed();
                time = Instant::now();

                let texts = ocr_engine.extract(
                    image,
                    resolution_multiplier,
                    (monitor.width().unwrap(), monitor.height().unwrap()),
                );

                let extract_time = time.elapsed();
                debug!(
                    "\nCapture time : {}ms\nExtract time : {}ms",
                    capture_time.as_millis(),
                    extract_time.as_millis()
                );

                app_handle
                    .emit(Events::NewTranslatedText.as_str(), texts)
                    .unwrap();
            }
        });

        self.is_running.store(true, Ordering::Release);
    }

    #[inline(always)]
    pub fn stop(&self) {
        self.need_stop.store(true, Ordering::Release);
    }

    #[inline(always)]
    pub fn update_config(&self, new_config: TranscendiaConfig) {
        let mut config = self.config.0.lock().unwrap();
        *config = new_config;
    }
}

/*fn translate_text(text: &mut String, target_lang: &str, client: &Client) {
                let client = Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(10))
                .https_only(true)
                .build()
                .expect("Could not create HTTP client");

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
