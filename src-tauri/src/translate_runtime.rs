use crate::config::Region;
use crate::events::Events;
use crate::ocr_models::{DETECTION_MODEL_NAME, MODEL_FOLDER_NAME, RECOGNITION_MODEL_NAME};
use image::DynamicImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use reqwest::{Client, Url};
use rten::Model;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_macos_permissions::{
    check_screen_recording_permission, request_screen_recording_permission,
};
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

pub struct TranslateRuntime {
    pub need_stop: Arc<Notify>,
    pub is_running: Arc<AtomicBool>,
    pub interval: Arc<AtomicU8>,
}

pub fn start_translate_runtime(
    app_handle: &AppHandle,
    data: &TranslateRuntime,
    monitor: u32,
    region: Region,
    lang: String,
) {
    let app_handle = app_handle.clone();
    let interval = data.interval.clone();
    let need_stop = data.need_stop.clone();

    if data.is_running.load(Ordering::Relaxed) {
        return;
    }

    tauri::async_runtime::spawn(async move {
        #[cfg(target_os = "macos")]
        if !check_screen_recording_permission().await {
            println!("No permission for screen capture !");
            request_screen_recording_permission().await;
            return;
        }

        let monitors = xcap::Monitor::all().unwrap();
        let monitor = monitors
            .iter()
            .find(|m| m.id().expect("Can't get monitor name") == monitor)
            .unwrap_or(monitors.get(0).expect("Cannot find any monitor"));

        let models_folder = app_handle
            .path()
            .app_config_dir()
            .expect("Could not get app config dir")
            .join(MODEL_FOLDER_NAME);

        let detection_model = Model::load_file(models_folder.join(DETECTION_MODEL_NAME))
            .expect("Could not load detection model");
        let recognition_model = Model::load_file(models_folder.join(RECOGNITION_MODEL_NAME))
            .expect("Could not load recognition model");

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })
            .expect("Impossible to create OCR engine");

        let mut old_text = String::new();

        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .https_only(true)
            .build()
            .expect("Could not create HTTP client");

        loop {
            tokio::select! {
                _ = need_stop.notified() => {
                    break;
                }
                _ = sleep(Duration::from_secs(interval.load(Ordering::Relaxed) as u64)) => {
                    println!("Translate runtime scheduling...");

                    let mut text = take_and_process_screenshot(monitor, &region, &engine);
                    if text == old_text {
                        println!("No changes in text");
                        continue;
                    } else {
                        old_text = text.clone();
                    }

                    translate_text(&mut text, &lang, &client).await;

                    app_handle.emit(Events::NewTranslatedText.as_str(), text).unwrap();

                    println!("End of runtime scheduling !");
                }
            }
        }
    });

    data.is_running.store(true, Ordering::Release);
}

pub fn stop_translate_runtime(data: &TranslateRuntime) {
    data.need_stop.notify_one();
    data.is_running.store(false, Ordering::Release);
}

fn take_and_process_screenshot(
    monitor: &xcap::Monitor,
    region: &Region,
    engine: &OcrEngine,
) -> String {
    let capture = monitor.capture_image().expect("Screen capture failed");
    let sf = monitor.scale_factor().expect("Can't get scale factor");
    let cropped_image = DynamicImage::ImageRgba8(capture)
        .crop_imm(
            (region.x as f32 * sf) as u32,
            (region.y as f32 * sf) as u32,
            (region.w as f32 * sf) as u32,
            (region.h as f32 * sf) as u32,
        )
        .to_rgb8();

    let img_source =
        ImageSource::from_bytes(cropped_image.as_raw(), cropped_image.dimensions()).unwrap();
    let ocr_input = engine
        .prepare_input(img_source)
        .expect("Could not prepare input for OCR");

    let world_rects = engine.detect_words(&ocr_input).unwrap();
    let line_rects = engine.find_text_lines(&ocr_input, &world_rects);
    let line_texts = engine
        .recognize_text(&ocr_input, &line_rects)
        .expect("Could not recognize text");

    let mut text_buffer = String::from("");
    for line in line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 1)
    {
        text_buffer.push_str(format!("{}\n", line).as_str());
    }

    text_buffer
}

async fn translate_text(text: &mut String, target_lang: &String, client: &Client) {
    println!("Translating text...");

    let mut url = Url::parse(
        format!(
            "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t",
            target_lang
        )
            .as_str(),
    )
        .unwrap();
    url.query_pairs_mut().append_pair("q", text);
    let res = client.get(url).send().await;

    if let Ok(r) = res {
        println!("URL: {}", r.url());
        let res_text = r.text().await.expect("Could not read response");

        let json = serde_json::from_str::<Value>(&res_text).expect("Could not parse json");
        if let Some(values) = json.get(0).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|i| i.get(0).and_then(|t| t.as_str()))
                .collect::<Vec<&str>>()
        }) {
            (*text).clear();
            for value in values {
                (*text).push_str(value);
            }
        } else {
            println!("Could not find translated text in response");
        }
    }
}
