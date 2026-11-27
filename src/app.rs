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
use crate::config::{SharedConfig, TranscendiaConfig};
use crate::events::UserEvent;
use crate::gfx::Gfx;
use crate::hotkeys::Hotkeys;
use crate::monitors::monitor_rect;
use crate::runtime::ocr::OcrResult;
use crate::runtime::runtime::TranscendiaRuntime;
use crate::tray::{self, Tray};
use crate::ui::windows;
use crate::ui::windows::{AppAction, WindowKind};
use crate::{monitors, paths, platform_specifics};
use global_hotkey::HotKeyState;
use log::{debug, error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::{WindowAttributes, WindowButtons, WindowId, WindowLevel};

pub struct AppState {
    pub config: SharedConfig,
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

impl App {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let config = Arc::new(Mutex::new(TranscendiaConfig::load()));
        let runtime = TranscendiaRuntime::new(Arc::clone(&config));

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
        let mut attrs: WindowAttributes = WindowAttributes::default()
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_resizable(false)
            .with_visible(false);
        let monitor = self.state.config.lock().unwrap().monitor;
        let monitor_rect = monitor_rect(monitor);

        match kind {
            WindowKind::Overlay => {
                attrs = attrs
                    .with_title("Transcendia - Overlay")
                    .with_transparent(true)
                    .with_decorations(false)
                    .with_content_protected(true)
                    .with_position(monitor_rect.0)
                    .with_inner_size(monitor_rect.1);

                attrs = platform_specifics::macos::no_shadow_and_first_mouse(attrs, false);
            }
            WindowKind::Config => {
                attrs = attrs
                    .with_title("Transcendia - Configuration")
                    .with_content_protected(true)
                    .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MAXIMIZE)
                    .with_inner_size(LogicalSize::new(400.0, 520.0));
            }
            WindowKind::Select => {
                attrs = attrs
                    .with_title("Transcendia - Select a region")
                    .with_transparent(true)
                    .with_decorations(false)
                    .with_position(monitor_rect.0)
                    .with_inner_size(monitor_rect.1);

                attrs = platform_specifics::macos::no_shadow_and_first_mouse(attrs, true);
            }
        }

        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(e) => {
                error!("Cannot create {kind:?} window: {e}");
                return;
            }
        };

        platform_specifics::macos::make_popup_window(&window);

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

    fn close_window(&mut self, kind: WindowKind) -> bool {
        if let Some(id) = self.find(kind) {
            self.windows.remove(&id);
            return true;
        }
        false
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

    pub fn close_overlay(&mut self) -> bool {
        if self.close_window(WindowKind::Overlay) {
            self.state.runtime.stop();
        } else {
            return false;
        }
        if let Some(t) = &self.tray {
            t.set_overlay_checked(false);
        }
        true
    }

    fn toggle_overlay(&mut self, event_loop: &ActiveEventLoop) {
        if self.find(WindowKind::Select).is_some() {
            return;
        }

        if !self.close_overlay() {
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
        debug!("Open select");
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
        if !self.close_window(WindowKind::Select) {
            return;
        };

        if let Some(h) = &mut self.hotkeys {
            h.unregister_finish();
        }
        self.open_overlay(event_loop);
        self.open_config(event_loop);
    }

    fn finish_select_region(&mut self, event_loop: &ActiveEventLoop) {
        self.close_select(event_loop);
    }

    fn apply_config(&mut self, monitor_changed: bool) {
        let config = self.state.config.lock().unwrap();
        config.save();
        let monitor = config.monitor;
        drop(config);
        if monitor_changed {
            if let Some(id) = self.find(WindowKind::Overlay) {
                let (pos, size) = monitors::monitor_rect(monitor);
                let w = &self.windows[&id].1.window;
                w.set_outer_position(pos);
                let _ = w.request_inner_size(size);
            }
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

    fn open_github(&self) {
        if let Err(_) = open::that("https://github.com/Xylobyte/transcendia") {
            error!("Cannot open GitHub page");
        }
    }

    fn handle_actions(&mut self, event_loop: &ActiveEventLoop, actions: Vec<AppAction>) {
        for action in actions {
            match action {
                AppAction::CloseWindow(WindowKind::Config) => self.close_config(),
                AppAction::CloseWindow(WindowKind::Select) => self.close_select(event_loop),
                AppAction::CloseWindow(WindowKind::Overlay) => {
                    self.close_overlay();
                }
                AppAction::OpenSelect => self.open_select(event_loop),
                AppAction::FinishSelectRegion => self.finish_select_region(event_loop),
                AppAction::ConfigChanged { monitor_changed } => self.apply_config(monitor_changed),
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
                tray::ID_OPEN_GITHUB => self.open_github(),
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

        match event {
            WindowEvent::CloseRequested => {
                match kind {
                    WindowKind::Config => self.close_config(),
                    WindowKind::Select => self.close_select(event_loop),
                    WindowKind::Overlay => {
                        self.close_overlay();
                    }
                }
                return;
            }
            WindowEvent::Resized(size) => gfx.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let actions = gfx.redraw(*kind != WindowKind::Config, |ui| {
                    windows::draw(*kind, ui, &mut self.state)
                });
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
