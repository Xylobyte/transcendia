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
import { computed, CSSProperties, onMounted, onUnmounted, ref } from "vue";
import { Config } from "../types/config.ts";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Events } from "../types/events.ts";
import { OcrResult } from "../types/translated-text.ts";

const config = ref<Config>();
const texts = ref<OcrResult[]>([]);

let unlistenRefresh: UnlistenFn;
let unlistenNewText: UnlistenFn;

onMounted(async () => {
	await getConfig();

	unlistenRefresh = await listen(Events.RefreshOverlay, getConfig);

	unlistenNewText = await listen(Events.NewTranslatedText, (event) => {
		texts.value = event.payload as OcrResult[];
		console.log(texts.value);
	});
});

onUnmounted(() => {
	unlistenRefresh();
	unlistenNewText();
});

const mainStyle = computed(() => {
	return {
		color: config.value?.text_color,
	} as CSSProperties;
});

const getConfig = async () => {
	config.value = await invoke<Config>("get_config");
};
</script>

<template>
	<main :style="mainStyle">
		<div
			v-for="text in texts"
			:key="`${text.x}x ${text.y}y ${text.width}w ${text.height}h`"
			:style="{
				top: text.y + (config?.region?.y || 0) + 'px',
				left: text.x + (config?.region?.x || 0) + 'px',
				minWidth: text.width + 'px',
				height: text.height + 'px',
			}"
			class="ct"
		>
			<span :style="{ fontSize: text.height / 2 + 'px' }">
				{{ text.text }}
			</span>
		</div>
		<div v-if="texts.length < 1" class="loading">Loading...</div>
	</main>
</template>

<style scoped>
main {
	position: relative;
	width: 100%;
	height: 100%;
}

.ct {
	display: flex;
	position: absolute;
	white-space: break-spaces;
	align-items: center;
	padding: 3px;
	background: rgba(0, 0, 0, 0.5);
	border-radius: 5px;
}

span {
	width: 100%;
	text-align: justify;
	text-align-last: justify;
	text-shadow: 0 0 5px rgba(0, 0, 0, 1);
	letter-spacing: 1px;
}

.loading {
	position: absolute;
	top: 50%;
	left: 50%;
	transform: translate(-50%, -50%);
	color: rgb(227, 227, 227);
	text-shadow: 0 0 5px rgba(0, 0, 0, 1);
	font-size: 50px;
}
</style>

<style>
html {
	background: transparent;
}

body {
	display: flex;
	background: transparent;
	align-items: center;
	justify-content: center;
}
</style>
