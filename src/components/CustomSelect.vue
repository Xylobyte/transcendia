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
    items: { value: string, label: string }[];
    defaultItem: string;
}>();

defineEmits<{
    itemChange: [value: string];
}>();
</script>

<template>
    <div class="select">
        <select @change="e => $emit('itemChange', (e.target as HTMLSelectElement).value)">
            <option
                v-for="item in $props.items"
                :key="item.value"
                :selected="$props.defaultItem === item.value"
                :value="item.value"
            >
                {{ item.label }}
            </option>
        </select>
    </div>
</template>

<style scoped>
select {
    appearance: none;
    background-color: transparent;
    border: none;
    padding: 6px 15px;
    margin: 0;
    width: 100%;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    z-index: 1;
    outline: none;
}

.select {
    display: grid;
    grid-template-areas: "select";
    align-items: center;
    position: relative;
    border: 1px solid white;
    border-radius: 6px;
    min-width: 150px;
    font-size: 1rem;
    background-color: transparent;
}

.select:hover {
    background-color: rgba(255, 255, 255, 0.1);
}

select, .select::after {
    grid-area: select;
}

.select::after {
    content: "";
    margin-right: 1em;
    justify-self: end;
    width: 0.8em;
    height: 0.5em;
    background-color: white;
    clip-path: polygon(100% 0%, 0 0%, 50% 100%);
}
</style>