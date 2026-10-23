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

/// (value, label) pairs, ported from `legacy-ui/src/ConfigWindow/languages.ts`.
pub const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("fr", "French"),
    ("es", "Spanish"),
    ("de", "German"),
    ("pt", "Portuguese (Portugal)"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("it", "Italian"),
    ("nl", "Dutch"),
    ("sv", "Swedish"),
    ("da", "Danish"),
    ("nb", "Norwegian Bokmål"),
    ("no", "Norwegian Nynorsk"),
    ("fi", "Finnish"),
    ("pl", "Polish"),
    ("cs", "Czech"),
    ("hu", "Hungarian"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("tr", "Turkish"),
    ("el", "Greek"),
    ("zh-CN", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("sl", "Slovenian"),
    ("sk", "Slovak"),
    ("bg", "Bulgarian"),
    ("lt", "Lithuanian"),
    ("lv", "Latvian"),
    ("vi", "Vietnamese"),
    ("th", "Thai"),
    ("id", "Indonesian"),
    ("ms", "Malay"),
    ("he", "Hebrew"),
    ("ar", "Arabic"),
    ("fa", "Persian (Farsi)"),
    ("hi", "Hindi"),
];

pub fn label_for(value: &str) -> &str {
    LANGUAGES
        .iter()
        .find(|(v, _)| *v == value)
        .map(|(_, l)| *l)
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_all_languages() {
        assert_eq!(LANGUAGES.len(), 37);
        assert_eq!(label_for("pt-BR"), "Portuguese (Brazil)");
        assert_eq!(label_for("xx"), "xx");
    }
}
