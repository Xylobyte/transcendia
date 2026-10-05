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
                    precision_mode: PrecisionMode::Low,
                    min_result_confidence: 0.7,
                    det_options: DetOptions {
                        box_border: 0,
                        min_area: 40,
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
        scale_factor: f32,
        box_threshold_x: i32,
        box_threshold_y: i32,
    ) -> Vec<OcrResult> {
        let result = self.engine.recognize(&image);
        match result {
            Ok(texts) => {
                let texts = self.cleanup_and_convert_texts(texts, scale_factor);
                // return texts;
                let blocks = self.merge_ocr_blocks(texts, false, box_threshold_x, box_threshold_y);
                blocks
            }
            Err(e) => {
                error!("Cannot detect text in image ! {}", e);
                Vec::new()
            }
        }
    }

    #[inline(always)]
    fn cleanup_and_convert_texts(
        &mut self,
        texts: Vec<OcrResult_>,
        scale_factor: f32,
    ) -> Vec<OcrResult> {
        let mut final_texts = Vec::new();
        for text in texts {
            let t = text.text.trim();
            if self.is_not_special_regex.is_match(t)
                && t.len() > 1
                && !t.chars().all(|c| c.is_ascii_digit())
            {
                final_texts.push(OcrResult {
                    text: t.to_string(),
                    x: (text.bbox.rect.left() as f32 / scale_factor) as i32,
                    y: (text.bbox.rect.top() as f32 / scale_factor) as i32,
                    width: (text.bbox.rect.width() as f32 / scale_factor) as u32,
                    height: (text.bbox.rect.height() as f32 / scale_factor) as u32,
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
        thresh_x: i32,
        thresh_y: i32,
    ) -> Vec<OcrResult> {
        if blocks.len() <= 1 {
            return blocks;
        }

        let line_y_tolerance = (thresh_y / 2).max(1);

        blocks.sort_unstable_by_key(|block| (block.y, block.x));

        let mut merged: Vec<OcrResult> = Vec::with_capacity(blocks.len());

        for next in blocks {
            let mut has_merged = false;

            for last in merged.iter_mut().rev() {
                let last_right = last.x + last.width as i32;
                let last_bottom = last.y + last.height as i32;
                let next_right = next.x + next.width as i32;
                let next_bottom = next.y + next.height as i32;

                let horizontal_overlap = (last_right.min(next_right) - last.x.max(next.x)).max(0);
                let vertical_overlap = (last_bottom.min(next_bottom) - next.y).max(0);

                let horizontal_gap = (next.x - last_right).max(last.x - next_right).max(0);
                let vertical_gap = (next.y - last_bottom).max(0);

                let same_line = (vertical_overlap * 3 >= last.height.min(next.height) as i32
                    || (next.y + next_bottom - last.y - last_bottom).abs() <= line_y_tolerance * 2)
                    && last.height.max(next.height)
                        <= (last.height.min(next.height) + (thresh_y as u32 * 3))
                    && (horizontal_overlap > 0 || horizontal_gap <= thresh_x);

                let same_column = horizontal_overlap * 2 >= last.width.min(next.width) as i32
                    || (next.x + next_right - last.x - last_right).abs()
                        <= last.width.min(next.width) as i32 + thresh_x * 2;

                let next_line = !same_line
                    && next.y > last.y
                    && (next.y + next_bottom - last.y - last_bottom).abs() > line_y_tolerance * 2
                    && vertical_gap <= thresh_y
                    && same_column;

                if same_line || next_line {
                    let next_text = next.text.trim();

                    if !next_text.is_empty() {
                        if is_japanese {
                            last.text.push_str(next_text);
                        } else if same_line {
                            if !last.text.is_empty() {
                                last.text.push(' ');
                            }
                            last.text.push_str(next_text);
                        } else if last.text.ends_with('-') {
                            last.text.pop();
                            last.text.push_str(next_text);
                        } else if matches!(next_text.chars().next(), Some('-' | '•' | '·'))
                            || (last.text.ends_with('.')
                                && next_text
                                    .chars()
                                    .next()
                                    .is_some_and(|c: char| char::is_ascii_uppercase(&c)))
                        {
                            last.text.push('\n');
                            last.text.push_str(next_text);
                        } else {
                            last.text.push(' ');
                            last.text.push_str(next_text);
                        }
                    }

                    let merged_right = last_right.max(next_right);
                    let merged_bottom = last_bottom.max(next_bottom);

                    last.x = last.x.min(next.x);
                    last.y = last.y.min(next.y);
                    last.width = (merged_right - last.x) as u32;
                    last.height = (merged_bottom - last.y) as u32;

                    if next_line {
                        last.line_count += 1;
                    }

                    has_merged = true;
                    break;
                }
            }

            if !has_merged {
                merged.push(next);
            }
        }

        merged
    }
}
