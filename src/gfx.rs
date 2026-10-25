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
use egui::ViewportId;
use egui_wgpu::{RendererOptions, WgpuConfiguration};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::window::Window;

/// Field order matters: the painter (surface) must drop before the window.
pub struct Gfx {
    pub painter: egui_wgpu::winit::Painter,
    pub state: egui_winit::State,
    pub ctx: egui::Context,
    pub window: Arc<Window>,
    pub repaint_at: Option<Instant>,
}

impl Gfx {
    pub fn new(window: Window, transparent: bool) -> Self {
        let window = Arc::new(window);
        let ctx = egui::Context::default();

        if transparent {
            ctx.all_styles_mut(|s| s.visuals.panel_fill = egui::Color32::TRANSPARENT);
        }

        let mut painter = pollster::block_on(egui_wgpu::winit::Painter::new(
            ctx.clone(),
            WgpuConfiguration::default(),
            transparent,
            RendererOptions::default(),
        ));
        if let Err(e) =
            pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window.clone())))
        {
            log::error!("Cannot create surface: {e}");
        }

        let state = egui_winit::State::new(
            ctx.clone(),
            ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            window.theme(),
            painter.max_texture_side(),
        );

        Self {
            painter,
            state,
            ctx,
            window,
            repaint_at: None,
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if let (Some(w), Some(h)) = (NonZeroU32::new(w), NonZeroU32::new(h)) {
            self.painter.on_window_resized(ViewportId::ROOT, w, h);
        }
    }

    pub fn redraw<R>(
        &mut self,
        transparent: bool,
        mut run_ui: impl FnMut(&mut egui::Ui) -> R,
    ) -> Option<R> {
        let mut result = None;
        let raw = self.state.take_egui_input(&self.window);
        let mut full = self.ctx.run_ui(raw, |ui| {
            result = Some(run_ui(ui));
        });

        self.state
            .handle_platform_output(&self.window, full.platform_output);

        let prims = self.ctx.tessellate(full.shapes, full.pixels_per_point);
        let clear = if transparent {
            [0.0; 4]
        } else {
            [0.1, 0.1, 0.1, 1.0]
        };
        self.painter.paint_and_update_textures(
            ViewportId::ROOT,
            full.pixels_per_point,
            clear,
            &prims,
            &mut full.textures_delta,
            Vec::new(),
            &self.window,
        );

        let delay = full
            .viewport_output
            .get(&ViewportId::ROOT)
            .map(|v| v.repaint_delay)
            .unwrap_or(Duration::MAX);
        self.repaint_at = if delay == Duration::MAX {
            None
        } else {
            Instant::now().checked_add(delay)
        };

        result
    }
}
