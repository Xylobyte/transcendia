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
use crate::runtime::ocr::{OcrGroupOrItem, OcrResult, TranscendiaOcrResults};
use log::{debug, error};
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

const SEPARATOR_CHARACTER: &str = "↪️";

pub struct TranscendiaTranslations {
    client: Client,
    translations_history: HashMap<String, String>,

    pub lang: String,
}

impl TranscendiaTranslations {
    #[inline]
    pub fn new<L: ToString>(lang: L) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(20))
                .https_only(true)
                .build()
                .expect("Could not create HTTP client"),
            translations_history: HashMap::new(),
            lang: lang.to_string(),
        }
    }

    pub fn translate(&mut self, texts: TranscendiaOcrResults) -> Vec<OcrResult> {
        let mut translated_texts = Vec::new();
        let mut need_translation_texts = Vec::new();

        for text in texts {
            match text {
                OcrGroupOrItem::Sentence(mut data) => {
                    let trad = self.translations_history.get(&data.text);
                    match trad {
                        Some(trad) => {
                            data.text = trad.clone();
                            translated_texts.push(data);
                        }
                        None => need_translation_texts.push(OcrGroupOrItem::Sentence(data)),
                    }
                }
                OcrGroupOrItem::Paragraph(data) => {
                    let mut need_translation_paragraph = Vec::with_capacity(data.len());

                    for mut item in data {
                        let trad = self.translations_history.get(&item.text);
                        match trad {
                            Some(trad) => {
                                item.text = trad.clone();
                                translated_texts.push(item);
                            }
                            None => need_translation_paragraph.push(item),
                        }
                    }

                    if !need_translation_texts.is_empty() {
                        need_translation_texts
                            .push(OcrGroupOrItem::Paragraph(need_translation_paragraph));
                    }
                }
            }
        }

        if !need_translation_texts.is_empty() {
            let new_translations = self
                .process_text(Self::build_string_to_translate(&need_translation_texts))
                .unwrap_or(Vec::new());
            debug!("Translations result: {:#?}", new_translations);

            let mut i = 0;
            for item in need_translation_texts {
                let sub_items = match item {
                    OcrGroupOrItem::Sentence(data) => vec![data],
                    OcrGroupOrItem::Paragraph(data) => data,
                };

                for mut sub_item in sub_items {
                    let trad = new_translations.get(i);
                    if let Some(trad) = trad {
                        self.translations_history
                            .insert(sub_item.text, trad.clone());

                        sub_item.text = trad.clone();
                        translated_texts.push(sub_item.clone());
                    }

                    i += 1;
                }
            }
        }

        translated_texts
    }

    #[inline(always)]
    fn process_text(&self, text: String) -> Result<Vec<String>, reqwest::Error> {
        let mut url = Url::parse(&format!(
            "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t", // Other option : https://github.com/ssut/py-googletrans/issues/268
            self.lang
        ))
        .unwrap();
        url.query_pairs_mut().append_pair("q", &text);

        let r = self.client.get(url).send()?;
        let res_text = r.text().expect("Could not read response");
        let json = serde_json::from_str::<Value>(&res_text).expect("Could not parse json");

        let mut texts = Vec::new();
        if let Some(values) = json.get(0).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|i| i.get(0).and_then(|t| t.as_str()))
                .collect::<Vec<&str>>()
        }) {
            for value in values {
                value.split(SEPARATOR_CHARACTER).for_each(|text| {
                    texts.push(text.trim().to_string());
                });
            }
        } else {
            error!("Could not find translated text in response");
        }

        Ok(texts)
    }

    #[inline(always)]
    fn build_string_to_translate(items: &TranscendiaOcrResults) -> String {
        let mut str_array = Vec::new();
        for item in items {
            match item {
                OcrGroupOrItem::Sentence(data) => str_array.push(data.text.clone()),
                OcrGroupOrItem::Paragraph(data) => {
                    let mut paragraph = Vec::with_capacity(data.len());
                    for d in data {
                        paragraph.push(d.text.clone());
                    }

                    str_array.push(paragraph.join(SEPARATOR_CHARACTER));
                }
            }
        }

        str_array.join("\n")
    }
}
