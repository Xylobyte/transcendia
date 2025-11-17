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
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Config } from "../types/config.ts";
import CustomButton from "../components/CustomButton.vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import CustomSelect from "../components/CustomSelect.vue";
import { ColorPicker } from "vue3-colorpicker";
import { languages } from "./languages.ts";

const currWindow = getCurrentWebviewWindow();

const monitors = ref<{ name: string; id: number }[]>([]);
const config = ref<Config>();
const canSave = ref(false);

onMounted(async () => {
	config.value = await invoke<Config>("get_config");
	monitors.value = await invoke("get_monitors");

	//document.addEventListener('contextmenu', event => event.preventDefault());
});

watch(
	() => [config.value?.text_color, config.value?.text_shadow_color],
	(_curr, prev) => {
		if (prev.every((v) => v === undefined)) return;
		canSave.value = true;
	},
);

const saveConfig = async (monitorChanged: boolean) => {
	try {
		await invoke<void>("set_config", {
			newConfig: config.value,
			reloadRuntime: true,
			monitorChanged,
		});
		canSave.value = false;
	} catch (e) {
		console.error(e);
	}
};

const changeMonitor = (monitor: string) => {
	if (!config.value) return;
	config.value.monitor = parseInt(monitor);
	saveConfig(true);
};

const changeLang = (lang: string) => {
	if (!config.value) return;
	config.value.lang = lang;
	saveConfig(false);
};

const onSelect = async () => {
	try {
		await invoke("select_region", {
			monitor: config.value?.monitor || monitors.value[0].id || "",
		});
		await currWindow.close();
	} catch (e) {
		console.error(e);
	}
};

const onToggleFullScreen = async () => {
	if (config.value?.region) {
		config.value.region = null;
		await saveConfig(false);
	} else {
		await onSelect();
	}
};
</script>

<template>
	<main v-if="config" ref="main">
		<h1>Configuration</h1>

		<div class="lang">
			<h2>Target language</h2>
			<CustomSelect
				:default-item="config.lang"
				:items="languages"
				@item-change="changeLang"
			/>
		</div>

		<div class="screen">
			<h2>Monitor</h2>
			<CustomSelect
				v-if="monitors.length > 0"
				:default-item="
					(config.monitor || monitors[0].id || 0).toString()
				"
				:items="
					monitors.map((m) => ({
						value: m.id.toString(),
						label: m.name || 'Monitor unnamed',
					}))
				"
				@item-change="changeMonitor"
			/>
		</div>

		<div class="full-screen">
			<h2>Translate the entire screen</h2>
			<input
				id="full-screen"
				:checked="!config.region"
				name="blur"
				type="checkbox"
				@change="onToggleFullScreen()"
			/>
		</div>

		<div v-if="config.region" class="region-select">
			<div class="head">
				<h2>Screen region</h2>
				<CustomButton
					:is-primary="false"
					title="Select region"
					@click="onSelect"
				>
					Select
				</CustomButton>
			</div>

			<div class="info">
				<span>X : {{ config?.region?.x || "?" }}</span>
				<span>Y : {{ config?.region?.y || "?" }}</span>
				<span>Width : {{ config?.region?.w || "?" }}</span>
				<span>Height : {{ config?.region?.h || "?" }}</span>
			</div>
		</div>

		<div class="text-color">
			<h2>Text color</h2>
			<ColorPicker
				v-model:pure-color="config.text_color"
				:z-index="20"
				format="rgb"
				lang="En"
				picker-type="chrome"
				theme="black"
			/>
		</div>

		<div class="bg-color">
			<h2>Text shadow color</h2>
			<ColorPicker
				v-model:pure-color="config.text_shadow_color"
				:disable-alpha="true"
				:z-index="20"
				format="rgb"
				lang="En"
				picker-type="chrome"
				theme="black"
			/>
		</div>
	</main>

	<div class="action">
		<CustomButton
			:disabled="!canSave"
			:is-primary="true"
			title="Close"
			@click="saveConfig(false)"
		>
			Save
		</CustomButton>
		<CustomButton
			:is-primary="true"
			title="Close"
			@click="currWindow.close()"
			>Close</CustomButton
		>
	</div>
</template>

<style scoped>
main {
	display: flex;
	flex-direction: column;
	padding: 10px;
	gap: 10px;
	width: 100%;
	height: 100%;
	overflow-y: scroll;
}

h1 {
	margin: 0;
	font-size: 1.6rem;
}

h2 {
	font-size: 1rem;
	font-weight: normal;
	color: rgb(174, 174, 174);
	margin: 0;
}

.region-select {
	display: flex;
	flex-direction: column;
	gap: 15px;
}

.region-select .info {
	color: rgb(174, 174, 174);
}

.region-select .head,
.screen,
.text-color,
.bg-color,
.lang,
.full-screen {
	display: flex;
	justify-content: space-between;
	align-items: center;
}

.screen,
.text-color,
.bg-color,
.region-select,
.lang,
.full-screen {
	background: #191919;
	padding: 10px;
	border-radius: 10px;
}

.region-select .info {
	display: flex;
	justify-content: space-between;
}

.action {
	display: flex;
	gap: 10px;
	padding: 10px;
	justify-content: end;
}
</style>

<style>
body {
	background: var(--background);
	display: flex;
	flex-direction: column;
}
</style>
