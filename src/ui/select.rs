use crate::app::AppState;
use crate::config::Region;
use crate::ui::{AppAction, WindowKind};
use egui::{Align2, Color32, CursorIcon, FontId, Pos2, Rect, Stroke, StrokeKind};

const MIN_SIZE: f32 = 50.0;

/// Normalizes a drag from `start` to `end` (any direction) into `(x, y, w, h)` in points.
pub fn normalize(start: Pos2, end: Pos2) -> (f32, f32, f32, f32) {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    (x, y, (end.x - start.x).abs(), (end.y - start.y).abs())
}

/// A region is kept only if both sides are strictly greater than 50.
pub fn is_valid_size(w: f32, h: f32) -> bool {
    w > MIN_SIZE && h > MIN_SIZE
}

/// Returns the region to save for a finished drag, if valid.
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
                let mut config = state.config.clone();
                config.region = Some(region);
                actions.push(AppAction::ApplyConfig {
                    config,
                    reload_runtime: true,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f32, y: f32) -> Pos2 {
        Pos2::new(x, y)
    }

    #[test]
    fn normalizes_all_directions() {
        for (a, b) in [
            (p(10.0, 20.0), p(110.0, 220.0)),
            (p(110.0, 220.0), p(10.0, 20.0)),
            (p(110.0, 20.0), p(10.0, 220.0)),
            (p(10.0, 220.0), p(110.0, 20.0)),
        ] {
            assert_eq!(normalize(a, b), (10.0, 20.0, 100.0, 200.0));
        }
    }

    #[test]
    fn strict_50_rule() {
        assert!(!is_valid_size(50.0, 100.0));
        assert!(!is_valid_size(100.0, 50.0));
        assert!(!is_valid_size(50.0, 50.0));
        assert!(is_valid_size(50.1, 50.1));
        assert!(region_from_drag(p(0.0, 0.0), p(50.0, 80.0)).is_none());
        let r = region_from_drag(p(100.0, 100.0), p(10.0, 20.0)).unwrap();
        assert_eq!((r.x, r.y, r.w, r.h), (10, 20, 90, 80));
    }
}
