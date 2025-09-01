<!--
  - Copyright © 2025 Nantsa Montillet
  - SPDX-License-Identifier: AGPL-3.0-or-later
  -
  - This program is free software: you can redistribute it and/or modify
  - it under the terms of the GNU Affero General Public License as published
  - by the Free Software Foundation, either version 3 of the License, or
  - (at your option) any later version.
  -
  - This program is distributed in the hope that it will be useful,
  - but WITHOUT ANY WARRANTY; without even the implied warranty of
  - MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  - GNU Affero General Public License for more details.
  -
  - You should have received a copy of the GNU Affero General Public License
  - along with this program.  If not, see <https://www.gnu.org/licenses/>.
  -->

<script lang="ts" setup>
defineProps<{
    title: string
    isPrimary: boolean
    disabled?: boolean
}>();
</script>

<template>
    <button
        :class="{primary: $props.isPrimary}"
        :disabled="$props.disabled"
        :title="$props.title"
    >
        <slot/>
    </button>
</template>

<style scoped>
button {
    display: flex;
    align-items: center;
    outline: none;
    border-radius: 6px;
    background: transparent;
    font-size: 1rem;
    border: 1px solid #d6d6d6;
    color: white;
    padding: 6px 15px;
}

button:disabled {
    pointer-events: none;
    opacity: 0.5;
}

button:hover {
    background: rgba(255, 255, 255, 0.1);
}

button.primary {
    background: var(--primary);
    color: var(--on-primary);
    border: none;
}

button.primary:hover {
    opacity: 0.8;
}
</style>