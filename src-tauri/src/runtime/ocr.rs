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
use log::error;
use rust_paddle_ocr::{Det, OcrError, Rec};
use serde::Serialize;
use std::collections::HashSet;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Clone, Debug)]
pub struct OcrResult {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    text: String,
}

impl OcrResult {
    #[inline]
    fn from(rect: Rect, resolution_multiplier: Option<f32>, text: String) -> Self {
        let factor = resolution_multiplier.unwrap_or(1.0);
        Self {
            x: (rect.left() as f32 / factor) as i32,
            y: (rect.top() as f32 / factor) as i32,
            width: (rect.width() as f32 / factor) as u32,
            height: (rect.height() as f32 / factor) as u32,
            text,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum OcrGroupOrItem {
    Sentence(OcrResult),
    Paragraph(Vec<OcrResult>),
}

pub type TranscendiaOcrResults = Vec<OcrGroupOrItem>;

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
                .with_rect_border_size(12),
            recognition: Rec::from_file(rec, keys)
                .expect("Could not load recognition model")
                .with_min_score(0.6)
                .with_punct_min_score(0.2),
        }
    }

    pub fn extract(
        &mut self,
        image: DynamicImage,
        resolution_multiplier: f32,
        box_threshold: i32,
    ) -> TranscendiaOcrResults {
        let result = self.detect(image);
        match result {
            Ok((text_rects, text_images)) => {
                let texts = self.recognize(text_images);

                Self::generate_results(texts, text_rects, resolution_multiplier, box_threshold)
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
            texts.push(text.trim().to_string());
        }
        texts
    }

    #[inline(always)]
    fn generate_results(
        texts: Vec<String>,
        text_rects: Vec<Rect>,
        resolution_multiplier: f32,
        box_threshold: i32,
    ) -> TranscendiaOcrResults {
        let mut skip: HashSet<usize> = Self::check_skip_items(&texts);
        let mut merged_ocr_results = Vec::<OcrResult>::new();
        for (i, actual_rect) in text_rects.iter().enumerate() {
            let text = texts[i].clone();
            if skip.contains(&i) {
                continue;
            }

            let mut actual_result =
                OcrResult::from(*actual_rect, Some(resolution_multiplier), text);
            skip.insert(i);

            let mut ii = 0;
            while ii < text_rects.len() {
                let text_ii = &texts[ii];
                if skip.contains(&ii) {
                    ii += 1;
                    continue;
                }

                let rect = Self::scale_rect(text_rects[ii], resolution_multiplier);

                if rect.top() > actual_result.y - box_threshold
                    && rect.bottom() < actual_result.y + actual_result.height as i32 + box_threshold
                {
                    if rect.left() < actual_result.x + actual_result.width as i32 + box_threshold
                        && rect.left() > actual_result.x
                    {
                        skip.insert(ii);
                        actual_result.width = (rect.right() - actual_result.x) as u32;
                        if !actual_result.text.ends_with(' ') && !text_ii.starts_with(' ') {
                            actual_result.text.push(' ');
                        }
                        actual_result.text.push_str(text_ii);
                        ii = 0;
                        continue;
                    }

                    if rect.right() < actual_result.x + actual_result.width as i32
                        && rect.right() > actual_result.x - box_threshold
                    {
                        skip.insert(ii);
                        actual_result.width =
                            (actual_result.x + actual_result.width as i32 - rect.left()) as u32;
                        actual_result.x = rect.left();
                        if !actual_result.text.starts_with(' ') && !text_ii.ends_with(' ') {
                            actual_result.text.insert(0, ' ');
                        }
                        actual_result.text.insert_str(0, text_ii);
                        ii = 0;
                        continue;
                    }
                }

                ii += 1;
            }

            merged_ocr_results.push(actual_result);
        }

        skip.clear();
        let mut results = TranscendiaOcrResults::new();
        for (i, mut ocr_result) in merged_ocr_results.iter().enumerate() {
            if skip.contains(&i) {
                continue;
            }

            let mut paragraphs: Vec<OcrResult> = vec![ocr_result.clone()];
            for ii in i + 1..merged_ocr_results.len() {
                let el = &merged_ocr_results[ii];
                if el.y < ocr_result.y + ocr_result.height as i32 + box_threshold
                    && el.y > ocr_result.y
                    && ((el.x - box_threshold <= ocr_result.x
                    && (el.x + box_threshold + el.width as i32)
                    >= ocr_result.x + ocr_result.width as i32)
                    || (el.x + box_threshold >= ocr_result.x
                    && (el.x - box_threshold + el.width as i32)
                    <= ocr_result.x + ocr_result.width as i32))
                {
                    skip.insert(ii);
                    paragraphs.push(el.clone());
                    ocr_result = el;
                }
            }

            results.push(if paragraphs.len() == 1 {
                OcrGroupOrItem::Sentence(paragraphs[0].clone())
            } else {
                OcrGroupOrItem::Paragraph(paragraphs)
            });
        }

        println!("{:#?}", results);
        results
    }

    #[inline(always)]
    fn check_skip_items(texts: &Vec<String>) -> HashSet<usize> {
        let mut results = HashSet::new();
        for (i, text) in texts.iter().enumerate() {
            if text.len() < 2 || text.parse::<f64>().is_ok() {
                results.insert(i);
            }
        }
        results
    }

    #[inline(always)]
    fn scale_rect(rect: Rect, resolution_multiplier: f32) -> Rect {
        Rect::at(
            (rect.left() as f32 / resolution_multiplier) as i32,
            (rect.top() as f32 / resolution_multiplier) as i32,
        )
            .of_size(
                (rect.width() as f32 / resolution_multiplier) as u32,
                (rect.height() as f32 / resolution_multiplier) as u32,
            )
    }
}
