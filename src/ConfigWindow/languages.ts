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

export const languages = [
    // Major Western Languages
    {value: 'en', label: 'English'},
    {value: 'es', label: 'Spanish'},
    {value: 'fr', label: 'French'},
    {value: 'de', label: 'German'},
    {value: 'it', label: 'Italian'},
    {value: 'pt', label: 'Portuguese (Portugal)'},
    {value: 'pt-br', label: 'Portuguese (Brazil)'},
    {value: 'nl', label: 'Dutch'},

    // Nordic Languages (Latin alphabet)
    {value: 'sv', label: 'Swedish'},
    {value: 'no', label: 'Norwegian'},
    {value: 'da', label: 'Danish'},
    {value: 'fi', label: 'Finnish'},

    // Central/Eastern European (Latin alphabet)
    {value: 'pl', label: 'Polish'},
    {value: 'cs', label: 'Czech'},
    {value: 'hu', label: 'Hungarian'},
    {value: 'ro', label: 'Romanian'},

    // Note: Russian uses Cyrillic, but keeping for existing users
    {value: 'ru', label: 'Russian (Limited OCR support)'},
]
