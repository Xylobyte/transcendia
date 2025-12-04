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
use crate::runtime::translations::TranscendiaTranslations;
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
        debug!("Starting transcendia runtime");

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
            let mut translation_engine = TranscendiaTranslations::new("en");

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
                let resolution_multiplier = m_lock.resolution_multiplier;
                translation_engine.lang = m_lock.lang.clone();
                drop(m_lock);

                let mut time = Instant::now();

                let image = monitor.capture_and_crop(resolution_multiplier, &region);

                let capture_time = time.elapsed();
                time = Instant::now();

                let threshold = (image.height() as f32 / 100.0) as i32;
                let texts = ocr_engine.extract(image, resolution_multiplier, threshold);

                let extract_time = time.elapsed();
                time = Instant::now();

                let texts = translation_engine.translate(texts);

                let translate_time = time.elapsed();
                debug!(
                    "\nCapture time : {}ms\nExtract time : {}ms\nTranslate time : {}ms",
                    capture_time.as_millis(),
                    extract_time.as_millis(),
                    translate_time.as_millis()
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
        debug!("Stopping transcendia runtime");
        self.need_stop.store(true, Ordering::Release);
    }

    #[inline(always)]
    pub fn update_config(&self, new_config: TranscendiaConfig) {
        let mut config = self.config.0.lock().unwrap();
        *config = new_config;
    }
}
