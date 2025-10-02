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
use log::error;
use rust_paddle_ocr::{Det, Rec};

pub struct TranscendiaOcr {
    detection: Det,
    recognition: Rec,
}

// SAFETY: We manually implement Send for TranscendiaOcr because:
// 1. The OCR engine will only be used within a single async task
// 2. We never actually send it between OS threads - tokio manages the async context
// 3. The raw pointers from MNN are only accessed from the same logical execution context
unsafe impl Send for TranscendiaOcr {}

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

    pub fn extract(&mut self, image: DynamicImage) -> String {
        let text_images = self.detection.find_text_img(&image);

        if let Ok(text_images) = text_images {
            for text_image in text_images {
                let text = self.recognition.predict_str(&text_image).expect("Cannot predict text");
                println!("Text: {:?}", text);
            }
        } else {
            error!("Could not detect text in image");
        }

        "".to_string()
    }

    fn detect(&mut self, image: DynamicImage) {}
}
