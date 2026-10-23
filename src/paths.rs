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
use directories::ProjectDirs;
use log::error;
use std::path::{Path, PathBuf};

pub fn config_dir() -> PathBuf {
    ProjectDirs::from("com", "xylobyte", "transcendia")
        .expect("Could not get app config dir")
        .config_dir()
        .to_path_buf()
}

fn models_candidates(exe: &Path, manifest_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(exe_dir) = exe.parent() {
        if let Some(contents) = exe_dir.parent() {
            v.push(contents.join("Resources").join("models"));
        }
        v.push(exe_dir.join("models"));
    }
    if let Some(m) = manifest_dir {
        v.push(m.join("models"));
    }
    v
}

fn pick_models_dir(candidates: &[PathBuf], exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    candidates.iter().find(|p| exists(p)).cloned()
}

pub fn models_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let candidates = models_candidates(&exe, Some(Path::new(env!("CARGO_MANIFEST_DIR"))));
    match pick_models_dir(&candidates, |p| p.is_dir()) {
        Some(p) => p,
        None => {
            error!("Models dir not found, tried: {:?}", candidates);
            candidates.into_iter().next().unwrap_or_default()
        }
    }
}
