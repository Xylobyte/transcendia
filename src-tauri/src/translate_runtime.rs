use crate::config::Region;
use crate::events::Events;
use crate::ocr_models::{DETECTION_MODEL_NAME, MODEL_FOLDER_NAME, RECOGNITION_MODEL_NAME};
use image::DynamicImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_macos_permissions::{check_screen_recording_permission, request_screen_recording_permission};
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
    monitor: String,
    region: Region,
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
            println!("No permission !");
            request_screen_recording_permission().await;
            return;
        }

        let monitors = xcap::Monitor::all().unwrap();
        let monitor = monitors
            .iter()
            .find(|m| m.name().expect("Can't get monitor name") == monitor)
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

        loop {
            tokio::select! {
                _ = need_stop.notified() => {
                    break;
                }
                _ = sleep(Duration::from_secs(interval.load(Ordering::Relaxed) as u64)) => {
                    println!("Translate runtime scheduling...");
                    let text = take_and_process_screenshot(monitor, &region, &engine);
                    app_handle.emit(Events::NewTranslatedText.as_str(), text).unwrap();
                    println!("End of runtime scheduling !")
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

pub fn take_and_process_screenshot(monitor: &xcap::Monitor, region: &Region, engine: &OcrEngine) -> String {
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
