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
	});
});

onUnmounted(() => {
	unlistenRefresh();
	unlistenNewText();
});

const mainStyle = computed(() => {
	return {
		color: config.value?.text_color,
		fontSize: "15px",
	} as CSSProperties;
});

const getConfig = async () => {
	config.value = await invoke<Config>("get_config");
};
</script>

<template>
	<main v-if="texts">
		<p :style="mainStyle">{{ texts }}</p>
	</main>
</template>

<style scoped>
main {
	border-radius: 30px;
	justify-content: center;
	display: flex;
	padding: 10px 20px;
}

p {
	width: 100%;
	white-space: break-spaces;
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
