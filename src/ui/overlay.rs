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
use crate::runtime::ocr::OcrResult;
use crate::ui::color::parse_color;
use egui::epaint::text::{LayoutJob, TextFormat};
use egui::{Align, Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};

const BOX_BG: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 178);
const PADDING: f32 = 3.0;
const RADIUS: u8 = 5;
const LOADING_COLOR: Color32 = Color32::from_rgb(227, 227, 227);

pub fn calc_font_size(width: f32, height: f32, text: &str) -> f32 {
    if text.is_empty() || width <= 0.0 || height <= 0.0 {
        return 12.0;
    }
    let usable_width = width * 0.95;
    let usable_height = height * 0.95;
    let paragraphs: Vec<usize> = text
        .split('\n')
        .map(|p| p.strip_suffix('\r').unwrap_or(p).chars().count())
        .collect();

    let fits = |size: f32| -> bool {
        let line_h = size * 1.2;
        let char_w = size * 0.6;
        let max_chars = usable_width / char_w;
        if max_chars < 1.0 {
            return false;
        }
        let total: f32 = paragraphs
            .iter()
            .map(|&len| ((len.max(1)) as f32 / max_chars).ceil())
            .sum();
        total * line_h <= usable_height
    };

    let mut low = 1i64;
    let mut high = usable_height.floor() as i64;
    let mut best = 1i64;
    while low <= high {
        let mid = (low + high) / 2;
        if fits(mid as f32) {
            best = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    best as f32
}

fn job(text: &str, size: f32, color: Color32, wrap: f32) -> LayoutJob {
    let mut job = LayoutJob::single_section(
        text.to_owned(),
        TextFormat {
            font_id: FontId::proportional(size),
            color,
            line_height: Some(size * 1.2),
            ..Default::default()
        },
    );
    job.wrap.max_width = wrap;
    job.justify = true;
    job.halign = Align::LEFT;
    job
}

fn paint_shadowed(ui: &egui::Ui, pos: Pos2, job: LayoutJob, shadow_job: LayoutJob) {
    let painter = ui.painter();
    let shadow = painter.layout_job(shadow_job);
    for (dx, dy) in [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5), (0.0, 0.0)] {
        painter.galley(pos + Vec2::new(dx, dy), shadow.clone(), Color32::BLACK);
    }
    let galley = painter.layout_job(job);
    painter.galley(pos, galley, Color32::WHITE);
}

fn shadow_color() -> Color32 {
    Color32::from_rgba_unmultiplied(0, 0, 0, 140)
}

fn draw_box(ui: &egui::Ui, t: &OcrResult, origin: Vec2, text_color: Color32) {
    let rect = Rect::from_min_size(
        Pos2::new(t.x as f32 + origin.x, t.y as f32 + origin.y),
        Vec2::new(t.width as f32, t.height as f32),
    );
    ui.painter().rect_filled(rect, RADIUS, BOX_BG);
    let size = calc_font_size(t.width as f32, t.height as f32, &t.text);
    let wrap = (rect.width() - 2.0 * PADDING).max(1.0);
    let galley = ui
        .painter()
        .layout_job(job(&t.text, size, text_color, wrap));
    let inner_h = rect.height() - 2.0 * PADDING;
    let y = rect.min.y + PADDING + ((inner_h - galley.size().y) / 2.0).max(0.0);
    let pos = Pos2::new(rect.min.x + PADDING, y);
    paint_shadowed(
        ui,
        pos,
        job(&t.text, size, text_color, wrap),
        job(&t.text, size, shadow_color(), wrap),
    );
}

pub fn draw(ui: &mut egui::Ui, state: &mut AppState) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            let config = &state.config;
            let text_color = parse_color(&config.text_color, Color32::WHITE);
            let origin = config
                .region
                .as_ref()
                .map(|r| Vec2::new(r.x as f32, r.y as f32))
                .unwrap_or(Vec2::ZERO);

            for t in &state.texts {
                draw_box(ui, t, origin, text_color);
            }

            if state.texts.is_empty() {
                let center = ui.max_rect().center();
                let font = FontId::proportional(50.0);
                let painter = ui.painter();
                for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
                    painter.text(
                        center + Vec2::new(dx, dy),
                        Align2::CENTER_CENTER,
                        "Loading...",
                        font.clone(),
                        shadow_color(),
                    );
                }
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    "Loading...",
                    font,
                    LOADING_COLOR,
                );
            }

            if config.show_fps {
                let painter = ui.painter();
                let galley = painter.layout_no_wrap(
                    format!("{:.2} tps", state.tps),
                    FontId::proportional(14.0),
                    text_color.gamma_multiply(0.5),
                );
                let rect = Rect::from_min_size(
                    Pos2::new(15.0, 15.0),
                    galley.size() + Vec2::splat(2.0 * PADDING),
                );
                painter.rect(
                    rect,
                    RADIUS,
                    Color32::BLACK.gamma_multiply(0.5),
                    Stroke::NONE,
                    StrokeKind::Inside,
                );
                painter.galley(rect.min + Vec2::splat(PADDING), galley, Color32::WHITE);
            }
        });
}
