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
    type?: "text" | "number";
    placeholder?: string;
    max?: number;
}>();

const model = defineModel();
</script>

<template>
    <input v-model="model" :max="$props.max" :placeholder="$props.placeholder" :type="$props.type">
</template>

<style scoped>
input {
    background: transparent;
    border: 1px solid white;
    border-radius: 6px;
    padding: 6px;
    width: 60px;
    text-align: center;
}
</style>