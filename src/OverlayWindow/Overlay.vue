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

const speed = ref(0);
let lastUpdate = Date.now();

let unlistenRefresh: UnlistenFn;
let unlistenNewText: UnlistenFn;

onMounted(async () => {
	await getConfig();

	unlistenRefresh = await listen(Events.RefreshOverlay, getConfig);

	unlistenNewText = await listen(Events.NewTranslatedText, (event) => {
		texts.value = event.payload as OcrResult[];

		const elapsed = Date.now() - lastUpdate;
		speed.value = 1000 / elapsed;
		lastUpdate = Date.now();
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

const calcFontSize = (text: OcrResult): string => {
	const { width, height, text: str } = text;
	if (!str || width <= 0 || height <= 0) return "12px";

	const usableWidth = width * 0.95;
	const usableHeight = height * 0.95;

	const paragraphs = str.split(/\r?\n/);

	const fits = (size: number): boolean => {
		const lineH = size * 1.2;
		const charW = size * 0.6;
		const maxCharsPerLine = usableWidth / charW;

		if (maxCharsPerLine < 1) return false;

		let totalLines = 0;
		for (const p of paragraphs) {
			const len = p.length || 1;
			totalLines += Math.ceil(len / maxCharsPerLine);
		}

		return totalLines * lineH <= usableHeight;
	};

	let low = 1;
	let high = Math.floor(usableHeight);
	let bestSize = 1;

	while (low <= high) {
		const mid = Math.floor((low + high) / 2);
		if (fits(mid)) {
			bestSize = mid;
			low = mid + 1;
		} else {
			high = mid - 1;
		}
	}

	return `${bestSize}px`;
};
</script>

<template>
	<main :style="mainStyle">
		<div v-if="config?.show_fps" class="ct fps">
			<span>{{ speed.toFixed(2) }} tps</span>
		</div>

		<div
			v-for="text in texts"
			:key="`${text.x}x ${text.y}y ${text.width}w ${text.height}h`"
			:style="{
				top: text.y + (config?.region?.y || 0) + 'px',
				left: text.x + (config?.region?.x || 0) + 'px',
				width: text.width + 'px',
				height: text.height + 'px',
			}"
			class="ct"
		>
			<span
				:style="{
					fontSize: calcFontSize(text),
				}"
			>
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
	background: rgba(0, 0, 0, 0.7);
	border-radius: 5px;
}

.fps {
	top: 15px;
	left: 15px;
	background: black;
	opacity: 0.5;
	z-index: 1000;
}

span {
	font-family: "Inter Tight", sans-serif;
	line-height: 1.2;
	width: 100%;
	text-align: justify;
	text-shadow: 0 0 5px rgba(0, 0, 0, 1);
	word-break: break-word;
	overflow-wrap: anywhere;
	hyphens: auto;
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
