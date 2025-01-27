use crate::events::Events;
use crate::windows::create_download_window;
use futures_util::StreamExt;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DownloadData {
    file: String,
    progress: u64,
    total_size: u64,
}

const MODEL_FOLDER_NAME: &str = "models";
const DETECTION_MODEL_NAME: &str = "text-detection.rten";
const DETECTION_MODEL_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";
const RECOGNITION_MODEL_NAME: &str = "text-recognition.rten";
const RECOGNITION_MODEL_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";

pub fn check_for_models(app: &AppHandle) -> bool {
    let conf_path = app
        .path()
        .app_config_dir()
        .expect("Could not get app config dir")
        .join(MODEL_FOLDER_NAME);
    if !conf_path.exists() {
        println!("{:?}", conf_path);
        fs::create_dir_all(&conf_path).expect("Could not create model dir");
    }

    let detection_model_path = conf_path.join(DETECTION_MODEL_NAME);
    let recognition_model_path = conf_path.join(RECOGNITION_MODEL_NAME);

    let mut its_ok = true;
    if !detection_model_path.exists() {
        its_ok = false;
        download_model(app, detection_model_path, DETECTION_MODEL_URL);
    }
    if !recognition_model_path.exists() {
        its_ok = false;
        download_model(app, recognition_model_path, RECOGNITION_MODEL_URL);
    }

    if !its_ok {
        create_download_window(app).expect("Cannot create download window");
    }

    its_ok
}

pub fn download_model(app: &AppHandle, path: PathBuf, url: &str) {
    let stop_notify = Arc::new(Notify::default());
    let stop_notify_clone = stop_notify.clone();
    let path_clone = path.clone();
    let id = app.listen(Events::StopDownload.as_str(), move |event| {
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
                    println!("Stop download and quit...");
                    app.exit(0);
                }
                data = stream.next() => {
                    match data {
                        Some(Ok(chunk)) => {
                            file.write_all(&chunk).await.expect("Error while writing file");
                            downloaded = min(downloaded + (chunk.len() as u64), total_size);
                            println!("Downloaded: {} / {}", downloaded, total_size);
                            app.emit(Events::DownloadProgress.as_str(), DownloadData {
                                file: url.clone(),
                                progress: downloaded.clone(),
                                total_size: total_size.clone(),
                            }).expect("Can't emit events");
                        }
                        Some(Err(err)) => {
                            eprintln!("Error while downloading: {}", err);
                            break;
                        }
                        None => {
                            println!("Download complete!");
                            break;
                        }
                    }
                }
            }
        }

        app.unlisten(id);
    });
}
