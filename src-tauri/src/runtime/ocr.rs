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
use crate::models::{EmbeddedModels, OCR_DET_FILE, OCR_KEYS_FILE, OCR_REC_FILE};
use image::DynamicImage;
use imageproc::rect::Rect;
use log::{debug, error};
use rust_paddle_ocr::{Det, OcrError, Rec};
use serde::Serialize;

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
    pub fn new() -> Self {
        let det = EmbeddedModels::get(OCR_DET_FILE).unwrap();
        let rec = EmbeddedModels::get(OCR_REC_FILE).unwrap();
        let keys = EmbeddedModels::get(OCR_KEYS_FILE).unwrap();

        Self {
            detection: Det::from_bytes(det.data.to_vec())
                .expect("Could not load detection model")
                .with_merge_boxes(false)
                .with_rect_border_size(12),
            recognition: Rec::from_bytes_with_keys(rec.data.to_vec(), keys.data.to_vec())
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
                    if t.is_empty() {
                        continue;
                    }

                    let merge_border = 35;
                    let merge_rect = results.iter_mut().find(|r| {
                        (rect.left() - r.x).abs() < merge_border
                            && (rect.top() - r.y).abs() < (r.height as i32 + merge_border)
                    });

                    match merge_rect {
                        Some(merge_rect) => {
                            merge_rect.height +=
                                (merge_rect.y - rect.top()).abs() as u32 + rect.height();
                            if rect.left() < merge_rect.x {
                                merge_rect.x = rect.left();
                            }
                            if rect.width() > merge_rect.width {
                                merge_rect.width = rect.width();
                            }
                            merge_rect.text.push('\n');
                            merge_rect.text.push_str(t);
                            merge_rect.line_height = merge_rect.height
                                / merge_rect.text.split('\n').collect::<Vec<_>>().len() as u32;
                        }
                        None => results.push(OcrResult {
                            x: rect.left(),
                            y: rect.top(),
                            width: rect.width(),
                            height: rect.height(),
                            line_height: rect.height()
                                / (&t).split('\n').collect::<Vec<&str>>().len() as u32,
                            text: t.to_string(),
                        }),
                    }
                }

                debug!("Texts: {:?}", results);
                results
            }
            Err(_) => {
                error!("Cannot detect text in image !");
                TranscendiaOcrResults::new()
            }
        }
    }

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
