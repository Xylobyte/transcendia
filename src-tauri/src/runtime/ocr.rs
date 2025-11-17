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
use imageproc::rect::Rect;
use log::{debug, error};
use rust_paddle_ocr::{Det, OcrError, Rec};
use serde::Serialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Clone, Debug)]
pub struct OcrResult {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    text: String,
    line_height: u32,
}

pub type TranscendiaOcrResults = Vec<OcrResult>;

pub struct TranscendiaOcr {
    detection: Det,
    recognition: Rec,
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
            detection: Det::from_file(det)
                .expect("Could not load detection model")
                .with_merge_boxes(false)
                .with_rect_border_size(10),
            recognition: Rec::from_file(rec, keys)
                .expect("Could not load recognition model")
                .with_min_score(0.6)
                .with_punct_min_score(0.2),
        }
    }

    pub fn extract(&mut self, image: DynamicImage) -> TranscendiaOcrResults {
        let result = self.detect(image);
        match result {
            Ok((text_rects, text_images)) => {
                let texts = self.recognize(text_images);
                let mut results = TranscendiaOcrResults::new();

                for (i, rect) in text_rects.iter().enumerate() {
                    let t = texts[i].trim();
                    if t.len() < 2 {
                        continue;
                    }

                    results.push(OcrResult {
                        x: rect.left(),
                        y: rect.top(),
                        width: rect.width(),
                        height: rect.height(),
                        line_height: rect.height()
                            / (&t).split('\n').collect::<Vec<&str>>().len() as u32,
                        text: t.to_string(),
                    })
                }

                results
            }
            Err(_) => {
                error!("Cannot detect text in image !");
                TranscendiaOcrResults::new()
            }
        }
    }

    #[inline(always)]
    fn detect(&mut self, image: DynamicImage) -> Result<(Vec<Rect>, Vec<DynamicImage>), OcrError> {
        let text_rects = self.detection.find_text_rect(&image)?;

        let mut images = Vec::<DynamicImage>::new();
        for text_rect in &text_rects {
            images.push(image.crop_imm(
                text_rect.left() as u32,
                text_rect.top() as u32,
                text_rect.width(),
                text_rect.height(),
            ))
        }

        Ok((text_rects, images))
    }

    #[inline(always)]
    fn recognize(&mut self, images: Vec<DynamicImage>) -> Vec<String> {
        let mut texts = Vec::<String>::new();
        for image in images {
            let text = self
                .recognition
                .predict_str(&image)
                .unwrap_or(String::new());
            texts.push(text);
        }
        texts
    }
}
