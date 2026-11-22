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
use crate::monitors::{BaseTranscendiaMonitor, TranscendiaMonitor};
use crate::ui::color::{format_color, parse_color};
use crate::ui::languages::{label_for, LANGUAGES};
use crate::ui::theme::{BG, CARD, MUTED, PRIMARY};
use crate::ui::windows::{AppAction, WindowKind};
use egui::color_picker::{color_edit_button_srgba, Alpha};
use egui::{Color32, CornerRadius, Frame, Margin, RichText};
use std::sync::Arc;
use std::time::{Duration, Instant};
use xcap::Monitor;

const DEBOUNCE: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct ConfigUi {
    monitors: Arc<[(u32, String)]>,
    multiplier: f32,
    pending_since: Option<Instant>,
    styled: bool,
}

fn load_monitors() -> Arc<[(u32, String)]> {
    Monitor::get_all()
        .unwrap_or_default()
        .into_iter()
        .map(|BaseTranscendiaMonitor { name, id }| {
            let name = if name.is_empty() {
                "Monitor unnamed".to_string()
            } else {
                name
            };
            (id, name)
        })
        .collect::<Vec<_>>()
        .into()
}

fn apply_style(ctx: &egui::Context) {
    ctx.all_styles_mut(|s| {
        let v = &mut s.visuals;
        v.dark_mode = true;
        v.panel_fill = BG;
        v.window_fill = BG;
        v.extreme_bg_color = CARD;
        v.override_text_color = Some(Color32::from_rgb(227, 227, 227));
        v.selection.bg_fill = PRIMARY;
        v.hyperlink_color = PRIMARY;
        for w in [
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
        ] {
            w.corner_radius = CornerRadius::same(6);
        }
        v.widgets.inactive.bg_fill = Color32::from_rgb(0x2c, 0x2c, 0x2c);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2c, 0x2c, 0x2c);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x38, 0x38, 0x38);
        v.widgets.active.weak_bg_fill = PRIMARY;
        v.widgets.active.bg_fill = PRIMARY;
    });
}

fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        .fill(CARD)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn row(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
        });
    });
}

fn apply(actions: &mut Vec<AppAction>, monitor_changed: bool) {
    actions.push(AppAction::ConfigChanged { monitor_changed });
}

pub fn draw(ui: &mut egui::Ui, state: &mut AppState) -> Vec<AppAction> {
    let mut actions = Vec::new();
    let id = ui.scope_id().with("config_ui");
    let mut config = state.config.lock().unwrap();
    let mut cui: ConfigUi = ui.data_mut(|d| {
        d.get_temp::<ConfigUi>(id).unwrap_or_else(|| ConfigUi {
            monitors: load_monitors(),
            multiplier: config.resolution_multiplier,
            pending_since: None,
            styled: false,
        })
    });
    if !cui.styled {
        apply_style(ui.ctx());
        cui.styled = true;
    }
    egui::CentralPanel::default()
        .frame(Frame::new().fill(BG).inner_margin(Margin::same(10)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.heading("Configuration");

                row(ui, "Target language", |ui| {
                    egui::ComboBox::from_id_salt("lang")
                        .selected_text(label_for(&config.lang))
                        .show_ui(ui, |ui| {
                            for (value, label) in LANGUAGES {
                                if ui.selectable_label(config.lang == *value, *label).clicked()
                                    && config.lang != *value
                                {
                                    config.lang = value.to_string();
                                    apply(&mut actions, false);
                                }
                            }
                        });
                });

                if !cui.monitors.is_empty() {
                    let current = if config.monitor == 0 {
                        cui.monitors[0].0
                    } else {
                        config.monitor
                    };
                    let current_name = cui
                        .monitors
                        .iter()
                        .find(|(i, _)| *i == current)
                        .map_or("Monitor unnamed", |(_, n)| n.as_str());
                    row(ui, "Monitor", |ui| {
                        egui::ComboBox::from_id_salt("monitor")
                            .selected_text(current_name)
                            .show_ui(ui, |ui| {
                                for (mid, name) in cui.monitors.iter() {
                                    if ui.selectable_label(current == *mid, name).clicked()
                                        && current != *mid
                                    {
                                        config.monitor = *mid;
                                        apply(&mut actions, true);
                                    }
                                }
                            });
                    });
                }

                let mut full = config.region.is_none();
                row(ui, "Translate the entire screen", |ui| {
                    if ui.checkbox(&mut full, "").changed() {
                        if full {
                            // checking: Vue opens Select instead of changing state
                            actions.push(AppAction::OpenSelect);
                        } else {
                            config.region = None;
                            apply(&mut actions, false);
                        }
                    }
                });

                if let Some(r) = &config.region {
                    card(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Screen region").color(MUTED));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Select").clicked() {
                                        actions.push(AppAction::OpenSelect);
                                    }
                                },
                            );
                        });
                        ui.horizontal(|ui| {
                            for (k, v) in [("X", r.x), ("Y", r.y), ("Width", r.w), ("Height", r.h)]
                            {
                                ui.label(RichText::new(format!("{k} : {v}")).color(MUTED));
                            }
                        });
                    });
                }
                row(ui, "Text color", |ui| {
                    let mut c = parse_color(&config.text_color, Color32::WHITE);
                    if color_edit_button_srgba(ui, &mut c, Alpha::OnlyBlend).changed() {
                        config.text_color = format_color(c);
                        apply(&mut actions, false);
                    }
                });

                row(ui, "Text background color", |ui| {
                    let mut c = parse_color(&config.text_shadow_color, Color32::BLACK);
                    if color_edit_button_srgba(ui, &mut c, Alpha::Opaque).changed() {
                        config.text_shadow_color = format_color(c);
                        apply(&mut actions, false);
                    }
                });

                row(ui, "Show translation speed in tps", |ui| {
                    if ui.checkbox(&mut config.show_fps, "").changed() {
                        apply(&mut actions, false);
                    }
                });

                card(ui, |ui| {
                    ui.label(
                        RichText::new(format!("Resolution multiplier ({}x)", cui.multiplier))
                            .color(MUTED),
                    );
                    ui.spacing_mut().slider_width = ui.available_width() - 60.0;
                    let r = ui.add(
                        egui::Slider::new(&mut cui.multiplier, 0.2..=2.0)
                            .step_by(0.1)
                            .max_decimals(1),
                    );
                    if r.changed() {
                        cui.pending_since = Some(Instant::now());
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    let close = egui::Button::new(RichText::new("Close").color(Color32::WHITE))
                        .fill(PRIMARY);
                    if ui.add(close).clicked() {
                        actions.push(AppAction::CloseWindow(WindowKind::Config));
                    }
                });
            });
        });

    if let Some(since) = cui.pending_since {
        let elapsed = since.elapsed();
        if elapsed >= DEBOUNCE {
            cui.pending_since = None;
            config.resolution_multiplier = (cui.multiplier * 10.0).round() / 10.0;
            apply(&mut actions, false);
        } else {
            ui.ctx().request_repaint_after(DEBOUNCE - elapsed);
        }
    }

    ui.data_mut(|d| d.insert_temp(id, cui));
    actions
}
