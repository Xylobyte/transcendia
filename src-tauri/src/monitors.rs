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
use crate::config::Region;
use crate::errors::TranscendiaError;
use image::{DynamicImage, RgbImage};
use serde::Serialize;
use xcap::Monitor;

#[derive(Debug, Serialize)]
pub struct BaseTranscendiaMonitor {
    name: String,
    id: u32,
}

pub trait TranscendiaMonitor {
    fn get_all() -> Result<Vec<BaseTranscendiaMonitor>, TranscendiaError>;
    fn load(id: u32) -> Self;
    fn capture_and_crop(&self, region: &Region) -> RgbImage;
}

impl TranscendiaMonitor for Monitor {
    fn get_all() -> Result<Vec<BaseTranscendiaMonitor>, TranscendiaError> {
        Monitor::all()
            .map(|ms| {
                ms.into_iter().map(|m| BaseTranscendiaMonitor {
                    name: m.name().unwrap(),
                    id: m.id().unwrap(),
                }).collect()
            })
            .map_err(|_| TranscendiaError::CannotGetMonitors)
    }

    fn load(id: u32) -> Self {
        let monitors = Monitor::all().unwrap();
        monitors
            .iter()
            .find(|m| m.id().expect("Can't get monitor name") == id)
            .unwrap_or(monitors.get(0).expect("Cannot find any monitor")).clone()
    }

    fn capture_and_crop(&self, region: &Region) -> RgbImage {
        let capture = self.capture_image().expect("Screen capture failed");
        let sf = self.scale_factor().expect("Can't get scale factor");
        DynamicImage::ImageRgba8(capture)
            .crop_imm(
                (region.x as f32 * sf) as u32,
                (region.y as f32 * sf) as u32,
                (region.w as f32 * sf) as u32,
                (region.h as f32 * sf) as u32,
            )
            .to_rgb8()
    }
}
