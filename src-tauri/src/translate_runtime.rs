extern crate objc;
use crate::config::Region;
use image::DynamicImage;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

#[cfg(target_os = "macos")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

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

    unsafe {
        #[cfg(target_os = "macos")]
        if !CGPreflightScreenCaptureAccess() {
            println!("Request screen access...");
            let result = CGRequestScreenCaptureAccess();
            println!("Screen access: {}", result);
        }
    }

    tauri::async_runtime::spawn(async move {
        let cache_dir = app_handle.path().app_cache_dir().unwrap();
        let monitors = xcap::Monitor::all().unwrap();
        let monitor = monitors
            .iter()
            .find(|m| m.name() == monitor)
            .unwrap_or(monitors.get(0).expect("Cannot find any monitor"));

        loop {
            tokio::select! {
                _ = need_stop.notified() => {
                    break;
                }
                _ = sleep(Duration::from_secs(interval.load(Ordering::Relaxed) as u64)) => {
                    println!("Translate runtime scheduling...");
                    take_and_process_screenshot(monitor, &region);
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

pub fn take_and_process_screenshot(monitor: &xcap::Monitor, region: &Region) {
    let capture = monitor.capture_image().expect("Screen capture failed");
    let cropped_image = DynamicImage::ImageRgba8(capture).crop_imm(region.x, region.y, region.w, region.h);
    cropped_image.save("target/test.png").expect("Cannot save image");
}
