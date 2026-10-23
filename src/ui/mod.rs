use crate::app::AppState;
use crate::config::TranscendiaConfig;

pub mod color;
pub mod config;
pub mod languages;
pub mod overlay;
pub mod select;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowKind {
    Overlay,
    Config,
    Select,
}

#[derive(Debug)]
pub enum AppAction {
    CloseWindow(WindowKind),
    OpenSelect,
    FinishSelectRegion,
    ApplyConfig {
        config: TranscendiaConfig,
        reload_runtime: bool,
        monitor_changed: bool,
    },
    Quit,
}

/// Placeholder UIs; real ones come in later steps.
pub fn draw(kind: WindowKind, ui: &mut egui::Ui, state: &mut AppState) -> Vec<AppAction> {
    let mut actions = Vec::new();
    match kind {
        WindowKind::Overlay => {
            overlay::draw(ui, state);
        }
        WindowKind::Config => {
            actions = config::draw(ui, state);
        }
        WindowKind::Select => {
            actions = select::draw(ui, state);
        }
    }
    actions
}
