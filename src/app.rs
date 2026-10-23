use crate::config::{ConfigState, TranscendiaConfig};
use crate::gfx::Gfx;
use crate::hotkeys::Hotkeys;
use crate::monitors::TranscendiaMonitor;
use crate::paths;
use crate::runtime::ocr::OcrResult;
use crate::runtime::runtime::TranscendiaRuntime;
use crate::tray::{self, Tray};
use crate::ui::{self, AppAction, WindowKind};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use log::{debug, error};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tray_icon::menu::MenuEvent;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::{WindowAttributes, WindowButtons, WindowId, WindowLevel};
use xcap::Monitor;

#[derive(Debug)]
pub enum UserEvent {
    Menu(MenuEvent),
    Hotkey(GlobalHotKeyEvent),
    NewTexts(Vec<OcrResult>),
}

pub struct AppState {
    pub config: TranscendiaConfig,
    pub texts: Vec<OcrResult>,
    pub tps: f32,
    pub last_update: Instant,
    pub runtime: TranscendiaRuntime,
}

pub struct App {
    proxy: EventLoopProxy<UserEvent>,
    state: AppState,
    windows: HashMap<WindowId, (WindowKind, Gfx)>,
    tray: Option<Tray>,
    hotkeys: Option<Hotkeys>,
    initialized: bool,
}

/// `monitor_id` is an xcap id, not a winit index.
pub fn monitor_rect(monitor_id: u32) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    let monitor = Monitor::load(monitor_id);
    let sf = monitor.scale_factor().unwrap_or(1.0) as f64;
    (
        LogicalPosition::new(
            monitor.x().unwrap_or(0) as f64 * sf,
            monitor.y().unwrap_or(0) as f64 * sf,
        ),
        LogicalSize::new(
            monitor.width().unwrap_or(800) as f64,
            monitor.height().unwrap_or(600) as f64,
        ),
    )
}

impl App {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let config = TranscendiaConfig::load();
        let runtime = TranscendiaRuntime::new(ConfigState(Mutex::new(config.clone())));
        Self {
            proxy,
            state: AppState {
                config,
                texts: Vec::new(),
                tps: 0.0,
                last_update: Instant::now(),
                runtime,
            },
            windows: HashMap::new(),
            tray: None,
            hotkeys: None,
            initialized: false,
        }
    }

    fn find(&self, kind: WindowKind) -> Option<WindowId> {
        self.windows
            .iter()
            .find(|(_, (k, _))| *k == kind)
            .map(|(id, _)| *id)
    }

    fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowKind) {
        use winit::platform::macos::WindowAttributesExtMacOS;
        let mut attrs: WindowAttributes = WindowAttributes::default()
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_resizable(false)
            .with_visible(false);
        let monitor_rect = monitor_rect(self.state.config.monitor);
        match kind {
            WindowKind::Overlay => {
                attrs = attrs
                    .with_title("Transcendia - Overlay")
                    .with_transparent(true)
                    .with_decorations(false)
                    .with_has_shadow(false)
                    .with_content_protected(true)
                    .with_position(monitor_rect.0)
                    .with_inner_size(monitor_rect.1);
            }
            WindowKind::Config => {
                attrs = attrs
                    .with_title("Transcendia - Configuration")
                    .with_accepts_first_mouse(true)
                    .with_content_protected(true)
                    .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MAXIMIZE)
                    .with_inner_size(LogicalSize::new(400.0, 650.0));
            }
            WindowKind::Select => {
                attrs = attrs
                    .with_title("Transcendia - Select a region")
                    .with_transparent(true)
                    .with_decorations(false)
                    .with_has_shadow(false)
                    .with_accepts_first_mouse(true)
                    .with_position(monitor_rect.0)
                    .with_inner_size(monitor_rect.1);
            }
        }
        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(e) => {
                error!("Cannot create {kind:?} window: {e}");
                return;
            }
        };
        crate::platform_specifics::macos::make_popup_window(&window);
        if kind == WindowKind::Overlay {
            if let Err(e) = window.set_cursor_hittest(false) {
                error!("Cannot set click-through: {e}");
            }
        }
        let transparent = kind != WindowKind::Config;
        let gfx = Gfx::new(window, transparent);
        gfx.window.set_visible(true);
        if kind != WindowKind::Overlay {
            gfx.window.focus_window();
        }
        gfx.window.request_redraw();
        self.windows.insert(gfx.window.id(), (kind, gfx));
    }

    fn close_window(&mut self, kind: WindowKind) {
        if let Some(id) = self.find(kind) {
            self.windows.remove(&id);
        }
    }

    pub fn open_overlay(&mut self, event_loop: &ActiveEventLoop) {
        if self.find(WindowKind::Overlay).is_some() {
            return;
        }
        self.open_window(event_loop, WindowKind::Overlay);
        let proxy = self.proxy.clone();
        self.state.runtime.start(paths::models_dir(), move |texts| {
            let _ = proxy.send_event(UserEvent::NewTexts(texts));
        });
        if let Some(t) = &self.tray {
            t.set_overlay_checked(true);
        }
    }

    pub fn close_overlay(&mut self) {
        if self.find(WindowKind::Overlay).is_some() {
            self.close_window(WindowKind::Overlay);
            self.state.runtime.stop();
        }
        if let Some(t) = &self.tray {
            t.set_overlay_checked(false);
        }
    }

    fn toggle_overlay(&mut self, event_loop: &ActiveEventLoop) {
        if self.find(WindowKind::Select).is_some() {
            return;
        }
        if self.find(WindowKind::Overlay).is_some() {
            self.close_overlay();
        } else {
            self.open_overlay(event_loop);
        }
    }

    pub fn open_config(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(id) = self.find(WindowKind::Config) {
            self.windows[&id].1.window.focus_window();
            return;
        }
        self.open_window(event_loop, WindowKind::Config);
        if let Some(t) = &self.tray {
            t.set_config_enabled(false);
        }
    }

    fn close_config(&mut self) {
        self.close_window(WindowKind::Config);
        if let Some(t) = &self.tray {
            t.set_config_enabled(true);
        }
    }

    fn open_select(&mut self, event_loop: &ActiveEventLoop) {
        if self.find(WindowKind::Select).is_some() {
            return;
        }
        self.close_overlay();
        self.open_window(event_loop, WindowKind::Select);
        if let Some(h) = &mut self.hotkeys {
            h.register_finish();
        }
        self.close_config();
    }

    fn close_select(&mut self, event_loop: &ActiveEventLoop) {
        if self.find(WindowKind::Select).is_none() {
            return;
        }
        self.close_window(WindowKind::Select);
        if let Some(h) = &mut self.hotkeys {
            h.unregister_finish();
        }
        self.open_overlay(event_loop);
        self.open_config(event_loop);
    }

    fn finish_select_region(&mut self, event_loop: &ActiveEventLoop) {
        self.state.runtime.update_config(self.state.config.clone());
        self.close_select(event_loop);
    }

    fn apply_config(
        &mut self,
        config: TranscendiaConfig,
        reload_runtime: bool,
        monitor_changed: bool,
    ) {
        config.save();
        self.state.config = config;
        if monitor_changed {
            if let Some(id) = self.find(WindowKind::Overlay) {
                let (pos, size) = monitor_rect(self.state.config.monitor);
                let w = &self.windows[&id].1.window;
                w.set_outer_position(pos);
                let _ = w.request_inner_size(size);
            }
        }
        if reload_runtime {
            self.state.runtime.update_config(self.state.config.clone());
        }
        self.request_redraw(WindowKind::Overlay);
    }

    fn request_redraw(&self, kind: WindowKind) {
        if let Some(id) = self.find(kind) {
            self.windows[&id].1.window.request_redraw();
        }
    }

    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        self.state.runtime.stop();
        self.hotkeys = None;
        self.windows.clear();
        self.tray = None;
        event_loop.exit();
    }

    fn handle_actions(&mut self, event_loop: &ActiveEventLoop, actions: Vec<AppAction>) {
        for action in actions {
            match action {
                AppAction::CloseWindow(WindowKind::Config) => self.close_config(),
                AppAction::CloseWindow(WindowKind::Select) => self.close_select(event_loop),
                AppAction::CloseWindow(WindowKind::Overlay) => self.close_overlay(),
                AppAction::OpenSelect => self.open_select(event_loop),
                AppAction::FinishSelectRegion => self.finish_select_region(event_loop),
                AppAction::ApplyConfig {
                    config,
                    reload_runtime,
                    monitor_changed,
                } => self.apply_config(config, reload_runtime, monitor_changed),
                AppAction::Quit => self.quit(event_loop),
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        match Tray::new() {
            Ok(t) => self.tray = Some(t),
            Err(e) => error!("Cannot create tray: {e}"),
        }
        self.hotkeys = Hotkeys::new();
        self.open_overlay(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Menu(e) => match e.id.as_ref() {
                tray::ID_CONFIG => self.open_config(event_loop),
                tray::ID_SHOW_OVERLAY => self.toggle_overlay(event_loop),
                tray::ID_QUIT => self.quit(event_loop),
                other => debug!("Menu item {other:?} not handled"),
            },
            UserEvent::Hotkey(e) => {
                if e.state != HotKeyState::Pressed {
                    return;
                }
                if let Some(h) = &self.hotkeys {
                    if e.id == h.toggle.id() {
                        self.toggle_overlay(event_loop);
                    } else if e.id == h.finish.id() && self.find(WindowKind::Select).is_some() {
                        self.finish_select_region(event_loop);
                    }
                }
            }
            UserEvent::NewTexts(texts) => {
                debug!("NewTexts: {} items", texts.len());
                let now = Instant::now();
                let elapsed = now.duration_since(self.state.last_update).as_secs_f32();
                if elapsed > 0.0 {
                    self.state.tps = 1.0 / elapsed;
                }
                self.state.last_update = now;
                self.state.texts = texts;
                self.request_redraw(WindowKind::Overlay);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some((kind, gfx)) = self.windows.get_mut(&id) else {
            return;
        };
        let kind = *kind;
        match event {
            WindowEvent::CloseRequested => {
                match kind {
                    WindowKind::Config => self.close_config(),
                    WindowKind::Select => self.close_select(event_loop),
                    WindowKind::Overlay => self.close_overlay(),
                }
                return;
            }
            WindowEvent::Resized(size) => gfx.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let state = &mut self.state;
                let actions =
                    gfx.redraw(kind != WindowKind::Config, |ui| ui::draw(kind, ui, state));
                if let Some(actions) = actions {
                    self.handle_actions(event_loop, actions);
                }
                return;
            }
            _ => {}
        }
        let response = gfx.state.on_window_event(&gfx.window, &event);
        if response.repaint {
            gfx.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let mut next: Option<Instant> = None;
        for (_, gfx) in self.windows.values_mut() {
            if let Some(t) = gfx.repaint_at {
                if t <= now {
                    gfx.repaint_at = None;
                    gfx.window.request_redraw();
                } else {
                    next = Some(next.map_or(t, |n| n.min(t)));
                }
            }
        }
        event_loop.set_control_flow(match next {
            Some(t) => ControlFlow::WaitUntil(t),
            None => ControlFlow::Wait,
        });
    }
}
