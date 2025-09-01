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
    progress: number,
    total: number,
}>();
</script>

<template>
    <div class="progress">
        <div :style="{width: `${$props.progress / $props.total * 100}%`}"></div>
    </div>
</template>

<style scoped>
.progress {
    border-radius: 5px;
    border: 1px solid #d6d6d6;
    background: transparent;
    overflow: hidden;
    width: 100%;
    height: 20px;

    div {
        height: 100%;
        width: 50px;
        background: var(--primary);
    }
}
</style>