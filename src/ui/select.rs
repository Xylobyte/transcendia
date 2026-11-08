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
use crate::app::AppState;
use crate::config::Region;
use crate::ui::windows::{AppAction, WindowKind};
use egui::{Align2, Color32, CursorIcon, FontId, Pos2, Rect, Stroke, StrokeKind};

const MIN_SIZE: f32 = 50.0;

pub fn normalize(start: Pos2, end: Pos2) -> (f32, f32, f32, f32) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    (x, y, (end.x - start.x).abs(), (end.y - start.y).abs())
}

pub fn is_valid_size(w: f32, h: f32) -> bool {
    w > MIN_SIZE && h > MIN_SIZE
}

pub fn region_from_drag(start: Pos2, end: Pos2) -> Option<Region> {
    let (x, y, w, h) = normalize(start, end);
    if !is_valid_size(w, h) {
        return None;
    }
    Some(Region {
        x: x.round() as u32,
        y: y.round() as u32,
        w: w.round() as u32,
        h: h.round() as u32,
    })
}

pub fn draw(ui: &mut egui::Ui, state: &mut AppState) -> Vec<AppAction> {
    let mut actions = Vec::new();
    let ctx = ui.ctx().clone();
    ctx.set_cursor_icon(CursorIcon::Crosshair);
    let drag_id = egui::Id::NULL.with("select_drag_start");

    let (pressed, released, pos) = ctx.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
            i.pointer.latest_pos(),
        )
    });
    let mut start: Option<Pos2> = ctx.data(|d| d.get_temp(drag_id));
    if pressed {
        start = pos;
        ctx.data_mut(|d| {
            match start {
                Some(s) => {
                    d.insert_temp(drag_id, s);
                }
                None => {
                    d.remove_temp::<Pos2>(drag_id);
                }
            };
        });
    }

    let screen = ctx.content_rect();
    let painter = ui.painter();
    let white = Color32::WHITE;
    let size = (screen.width() * 0.02).max(14.0);
    painter.text(
        screen.center() - egui::vec2(0.0, 15.0),
        Align2::CENTER_CENTER,
        "Select a capture region on the screen",
        FontId::proportional(size),
        white.gamma_multiply(0.5),
    );
    painter.text(
        screen.center() + egui::vec2(0.0, 15.0),
        Align2::CENTER_CENTER,
        "(Ctrl + X to cancel)",
        FontId::proportional(size * 0.75),
        white.gamma_multiply(0.5),
    );

    if let (Some(s), Some(e)) = (start, pos) {
        let (x, y, w, h) = normalize(s, e);
        let rect = Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h));
        painter.rect(
            rect,
            5.0,
            Color32::from_white_alpha(128),
            Stroke::new(2.0, white),
            StrokeKind::Inside,
        );
        if released {
            ctx.data_mut(|d| d.remove_temp::<Pos2>(drag_id));
            if let Some(region) = region_from_drag(s, e) {
                state.config.lock().unwrap().region = Some(region);
                actions.push(AppAction::ConfigChanged {
                    monitor_changed: false,
                });
                actions.push(AppAction::FinishSelectRegion);
                actions.push(AppAction::CloseWindow(WindowKind::Select));
            }
        }
        ctx.request_repaint();
    } else if released {
        ctx.data_mut(|d| d.remove_temp::<Pos2>(drag_id));
    }
    actions
}
