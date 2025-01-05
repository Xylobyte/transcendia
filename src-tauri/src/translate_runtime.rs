use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

pub struct TranslateRuntime {
    pub need_stop: Arc<Notify>,
    pub is_running: Arc<Mutex<bool>>,
}

pub fn start_translate_runtime(app_handle: &AppHandle, interval: u8) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        println!("Translate runtime started");

        loop {
            println!("Translate runtime interval");
            sleep(Duration::from_secs(interval as u64)).await;
        }
    });
}
