use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

pub struct TranslateRuntime {
    pub need_stop: Arc<Notify>,
    pub is_running: Arc<AtomicBool>,
    pub interval: Arc<AtomicU8>,
}

pub fn start_translate_runtime(app_handle: &AppHandle, data: &TranslateRuntime) {
    let app_handle = app_handle.clone();
    let interval = data.interval.clone();
    let need_stop = data.need_stop.clone();

    if data.is_running.load(Ordering::Relaxed) {
        return;
    }

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                _ = need_stop.notified() => {
                    break;
                }
                _ = sleep(Duration::from_secs(interval.load(Ordering::Relaxed) as u64)) => {
                    println!("Translation interval");
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
