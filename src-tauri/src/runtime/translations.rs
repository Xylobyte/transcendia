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
use crate::runtime::ocr::OcrResult;
use log::debug;
use std::collections::HashMap;

const SEPARATOR_CHARACTER: &str = "↪️";

pub struct TranscendiaTranslations {
    translations_history: HashMap<String, String>,
    pub lang: String,
}

impl TranscendiaTranslations {
    #[inline]
    pub fn new<L: ToString>(lang: L) -> Self {
        Self {
            translations_history: HashMap::new(),
            lang: lang.to_string(),
        }
    }

    pub fn translate(&mut self, texts: Vec<String>) -> Vec<OcrResult> {
        debug!("{:#?}", texts);

        let mut translated_texts = Vec::new();

        translated_texts
    }
}
