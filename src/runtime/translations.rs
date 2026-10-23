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
use crate::models::CTRANSLATE_TOK;
use crate::runtime::ocr::OcrResult;
use ct2rs::sys::Translator;
use ct2rs::{ComputeType, Config, TranslationOptions};
use log::debug;
use std::collections::HashMap;
use std::path::Path;
use tokenizers::Tokenizer;

pub struct TranscendiaTranslations {
    translations_history: HashMap<String, String>,
    tokenizer: Tokenizer,
    translator: Translator,
    pub source_lang: String,
    pub target_lang: String,
}

impl TranscendiaTranslations {
    pub fn new<L: ToString>(models_dir: &Path, src_lang: L, target_lang: L) -> Self {
        let tok = models_dir.join(CTRANSLATE_TOK);
        let model = models_dir.to_path_buf();

        Self {
            translations_history: HashMap::new(),
            tokenizer: Tokenizer::from_file(tok).unwrap(),
            translator: Translator::new(
                model,
                &Config {
                    compute_type: ComputeType::INT8,
                    num_threads_per_replica: 4,
                    ..Default::default()
                },
            )
            .unwrap(),
            source_lang: src_lang.to_string(),
            target_lang: target_lang.to_string(),
        }
    }

    pub fn translate(&mut self, mut texts: Vec<OcrResult>) -> Vec<OcrResult> {
        let texts_len = texts.len();
        if texts_len == 0 {
            return Vec::new();
        }

        let mut batch_tokens = Vec::with_capacity(texts_len);
        for text in texts.clone() {
            let Ok(encoding) = self.tokenizer.encode(text.text, false) else {
                return Vec::new();
            };
            let mut tokens = Vec::with_capacity(encoding.get_tokens().len() + 2);
            tokens.push(self.source_lang.clone());
            tokens.extend_from_slice(encoding.get_tokens());
            tokens.push("</s>".to_string());
            batch_tokens.push(tokens);
        }

        let target_prefix = vec![vec![self.target_lang.clone()]; texts_len];

        let Ok(results) = self.translator.translate_batch_with_target_prefix(
            &batch_tokens,
            &target_prefix,
            &TranslationOptions {
                beam_size: 1,
                repetition_penalty: 1.1,
                no_repeat_ngram_size: 3,
                max_decoding_length: 128,
                ..Default::default()
            },
            None,
        ) else {
            return Vec::new();
        };

        for (i, result) in results.iter().enumerate() {
            let Some(tokens) = result.output() else {
                continue;
            };

            let ids: Vec<u32> = tokens
                .iter()
                .filter(|t| *t != &self.target_lang)
                .filter_map(|t| self.tokenizer.token_to_id(t))
                .collect();

            match self.tokenizer.decode(&ids, true) {
                Ok(decoded) => {
                    texts[i].text = decoded;
                }
                Err(e) => debug!("Decode error: {}", e),
            }
        }

        texts
    }
}
