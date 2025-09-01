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
use crate::events::Events;
use crate::windows::create_download_window;
use futures_util::StreamExt;
use log::{debug, error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fs;
use std::fs::remove_file;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Notify;

pub const MODEL_FOLDER_NAME: &str = "ocr_models";
pub const DETECTION_MODEL_NAME: &str = "text-detection.mnn";
const DETECTION_MODEL_URL: &str =
    "https://github.com/zibo-chen/rust-paddle-ocr/raw/refs/heads/main/models/PP-OCRv5_mobile_det_fp16.mnn";
pub const RECOGNITION_MODEL_NAME: &str = "text-recognition.mnn";
const RECOGNITION_MODEL_URL: &str =
    "https://github.com/zibo-chen/rust-paddle-ocr/raw/refs/heads/main/models/PP-OCRv5_mobile_rec_fp16.mnn";
pub const OCR_KEYS_NAME: &str = "ocr_keys.txt";
const OCR_KEYS_URL: &str =
    "https://github.com/zibo-chen/rust-paddle-ocr/raw/refs/heads/main/models/ppocr_keys_v5.txt";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DownloadData {
    file: String,
    progress: u64,
    total_size: u64,
}

pub struct TranscendiaOcr {}

impl TranscendiaOcr {
    pub fn check(app: &AppHandle) -> bool {
        let conf_path = app
            .path()
            .app_config_dir()
            .expect("Could not get app config dir")
            .join(MODEL_FOLDER_NAME);
        if !conf_path.exists() {
            debug!("{:?}", conf_path);
            fs::create_dir_all(&conf_path).expect("Could not create model dir");
        }

        let detection_model_path = conf_path.join(DETECTION_MODEL_NAME);
        let recognition_model_path = conf_path.join(RECOGNITION_MODEL_NAME);
        let keys_path = conf_path.join(OCR_KEYS_NAME);

        let mut its_ok = true;
        if !detection_model_path.exists() {
            its_ok = false;
            Self::download_model(app, detection_model_path, DETECTION_MODEL_URL);
        }
        if !recognition_model_path.exists() {
            its_ok = false;
            Self::download_model(app, recognition_model_path, RECOGNITION_MODEL_URL);
        }
        if !keys_path.exists() {
            its_ok = false;
            Self::download_model(app, keys_path, OCR_KEYS_URL);
        }

        if !its_ok {
            create_download_window(app).expect("Cannot create download window");
        }

        its_ok
    }

    fn download_model(app: &AppHandle, path: PathBuf, url: &str) {
        let stop_notify = Arc::new(Notify::default());
        let stop_notify_clone = stop_notify.clone();
        let path_clone = path.clone();
        let id = app.listen(Events::StopDownload.as_str(), move |_| {
            let _ = remove_file(&path_clone);
            stop_notify.notify_one();
        });

        let app = app.clone();
        let url = url.to_string().clone();
        tauri::async_runtime::spawn(async move {
            let client = Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .https_only(true)
                .timeout(Duration::from_secs(20))
                .build()
                .expect("Cannot create https client!");

            let res = client
                .get(url.clone())
                .send()
                .await
                .expect(format!("Failed to GET from '{}'", url.clone()).as_str());

            let total_size = res.content_length().expect("Failed to get content length");
            let mut file = File::create(&path)
                .await
                .expect("Failed to create the file!");
            let mut downloaded: u64 = 0;
            let mut stream = res.bytes_stream();

            loop {
                tokio::select! {
                    _ = stop_notify_clone.notified() => {
                        info!("Stop download and quit runtime...");
                        app.exit(0);
                    }
                    data = stream.next() => {
                        match data {
                            Some(Ok(chunk)) => {
                                file.write_all(&chunk).await.expect("Error while writing file");
                                downloaded = min(downloaded + (chunk.len() as u64), total_size);
                                info!("Downloaded: {} / {}", downloaded, total_size);
                                app.emit(Events::DownloadProgress.as_str(), DownloadData {
                                    file: url.clone(),
                                    progress: downloaded.clone(),
                                    total_size: total_size.clone(),
                                }).expect("Can't emit events");
                            }
                            Some(Err(err)) => {
                                error!("Error while downloading: {}", err);
                                let err = tokio::fs::remove_file(path).await.is_err();
                                if err { error!("Can't remove file!"); }
                                break;
                            }
                            None => {
                                info!("Download complete!");
                                break;
                            }
                        }
                    }
                }
            }

            app.unlisten(id);
        });
    }
}
