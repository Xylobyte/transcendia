#![allow(dead_code)]

mod app;
mod config;
mod errors;
mod gfx;
mod hotkeys;
mod models;
mod monitors;
mod paths;
mod platform_specifics;
mod runtime;
mod tray;
mod ui;

use app::{App, UserEvent};
use global_hotkey::GlobalHotKeyEvent;
use single_instance::SingleInstance;
use tray_icon::menu::MenuEvent;
use winit::event_loop::EventLoop;
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

fn main() {
    env_logger::init();

    let guard = SingleInstance::new("com.xylobyte.transcendia")
        .expect("Cannot create single instance guard");
    if !guard.is_single() {
        log::warn!("Transcendia is already running");
        return;
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_activation_policy(ActivationPolicy::Accessory)
        .build()
        .expect("Cannot create event loop");
    let proxy = event_loop.create_proxy();

    let p = proxy.clone();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = p.send_event(UserEvent::Menu(e));
    }));
    let p = proxy.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |e| {
        let _ = p.send_event(UserEvent::Hotkey(e));
    }));

    let mut app = App::new(proxy);
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Event loop error: {e}");
    }
}
