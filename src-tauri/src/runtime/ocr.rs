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
use crate::models::{OCR_DET_FILE, OCR_KEYS_FILE, OCR_REC_FILE};
use image::DynamicImage;
use log::{debug, error};
use ocr_rs::{
    Backend, DetOptions, OcrEngine, OcrEngineConfig, OcrResult_, PrecisionMode, RecOptions,
};
use regex::Regex;
use serde::Serialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Clone, Debug)]
pub struct OcrResult {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub text: String,
    pub line_count: u32,
}

pub struct TranscendiaOcr {
    engine: OcrEngine,
    is_not_special_regex: Regex,
}

impl TranscendiaOcr {
    pub fn new(app_handle: &AppHandle) -> Self {
        let det = app_handle
            .path()
            .resolve(format!("models/{}", OCR_DET_FILE), BaseDirectory::Resource)
            .unwrap();
        let rec = app_handle
            .path()
            .resolve(format!("models/{}", OCR_REC_FILE), BaseDirectory::Resource)
            .unwrap();
        let keys = app_handle
            .path()
            .resolve(format!("models/{}", OCR_KEYS_FILE), BaseDirectory::Resource)
            .unwrap();

        Self {
            engine: OcrEngine::new(
                det,
                rec,
                keys,
                Some(OcrEngineConfig {
                    backend: Backend::CPU,
                    thread_count: 4,
                    precision_mode: PrecisionMode::High,
                    min_result_confidence: 0.94,
                    det_options: DetOptions {
                        box_border: 0,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            )
            .expect("Failed to init OCR engine!"),
            is_not_special_regex: Regex::new(r"[\p{L}\p{N}\d]").unwrap(),
        }
    }

    pub fn extract(
        &mut self,
        image: DynamicImage,
        resolution_multiplier: f32,
        box_threshold: i32,
    ) -> Vec<OcrResult> {
        let result = self.engine.recognize(&image);
        match result {
            Ok(texts) => {
                let texts = self.cleanup_and_convert_texts(texts);
                let blocks = self.merge_ocr_blocks(texts, false, 6, 6);
                blocks
            }
            Err(_) => {
                error!("Cannot detect text in image !");
                Vec::new()
            }
        }
    }

    #[inline(always)]
    fn cleanup_and_convert_texts(&mut self, texts: Vec<OcrResult_>) -> Vec<OcrResult> {
        let mut final_texts = Vec::new();
        for text in texts {
            let t = text.text.trim();
            if self.is_not_special_regex.is_match(t)
                && t.len() > 1
                && !t.chars().all(|c| c.is_ascii_digit())
            {
                final_texts.push(OcrResult {
                    text: t.to_string(),
                    x: text.bbox.rect.left(),
                    y: text.bbox.rect.top(),
                    width: text.bbox.rect.width(),
                    height: text.bbox.rect.height(),
                    line_count: 1,
                });
            }
        }
        final_texts
    }

    #[inline(always)]
    fn merge_ocr_blocks(
        &mut self,
        mut blocks: Vec<OcrResult>,
        is_japanese: bool,
        thresh_y: i32,
        thresh_x: i32,
    ) -> Vec<OcrResult> {
        if blocks.is_empty() {
            return vec![];
        }

        blocks.sort_unstable_by(|a, b| {
            let y_diff = a.y - b.y;
            if y_diff.abs() < 10 {
                a.x.cmp(&b.x)
            } else {
                a.y.cmp(&b.y)
            }
        });

        let mut merged = Vec::with_capacity(blocks.len());
        let mut iter = blocks.into_iter();
        merged.push(iter.next().unwrap());

        for next in iter {
            let last = merged.last_mut().unwrap();

            let last_left = last.x;
            let last_right = last_left + last.width as i32;
            let last_top = last.y;
            let last_bottom = last_top + last.height as i32;

            let next_left = next.x;
            let next_right = next_left + next.width as i32;
            let next_top = next.y;
            let next_bottom = next_top + next.height as i32;

            let vertical_gap = next_top - last_bottom;
            let horizontal_gap = next_left - last_right;
            let top_diff = (next_top - last_top).abs();

            let x_overlap = last_right.min(next_right) - last_left.max(next_left);
            let same_column = x_overlap > 10 || (next_left - last_left).abs() < 35;

            let is_same_line = top_diff <= 12 && horizontal_gap >= -15 && horizontal_gap < thresh_x;

            let is_next_line =
                top_diff > 12 && vertical_gap >= -15 && vertical_gap < thresh_y && same_column;

            if is_same_line || is_next_line {
                let next_text = next.text.trim();
                if !next_text.is_empty() {
                    if is_japanese {
                        last.text.push_str(next_text);
                    } else if is_same_line {
                        last.text.push(' ');
                        last.text.push_str(next_text);
                    } else if last.text.ends_with('-') {
                        last.text.pop();
                        last.text.push_str(next_text);
                    } else {
                        last.text.push(' ');
                        last.text.push_str(next_text);
                    }
                }

                let min_x = last_left.min(next_left);
                let min_y = last_top.min(next_top);
                let max_x = last_right.max(next_right);
                let max_y = last_bottom.max(next_bottom);

                last.x = min_x;
                last.y = min_y;
                last.width = (max_x - min_x) as u32;
                last.height = (max_y - min_y) as u32;
                if is_next_line {
                    last.line_count += 1;
                }
            } else {
                merged.push(next);
            }
        }

        merged
    }
}
