/*
 * Copyright © 2026 Nantsa Montillet
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
use crate::config::TranscendiaConfig;
use crate::ui::{config, overlay, select};

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
