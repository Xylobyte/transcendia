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
import {onMounted, ref, watch} from "vue";
import {invoke} from "@tauri-apps/api/core";
import {Config} from "../types/config.ts";
import CustomButton from "../components/CustomButton.vue";
import {exit} from "@tauri-apps/plugin-process";
import {getCurrentWebviewWindow} from "@tauri-apps/api/webviewWindow";
import {emit} from "@tauri-apps/api/event";
import {Events} from "../types/events.ts";
import CustomSelect from "../components/CustomSelect.vue";
import {ColorPicker} from "vue3-colorpicker";
import CustomInput from "../components/CustomInput.vue";
import {languages} from "./languages.ts";

const currWindow = getCurrentWebviewWindow();

const monitors = ref<{ name: string, id: number }[]>([]);
const config = ref<Config>();
const canSave = ref(false);

onMounted(async () => {
    await emit(Events.OnOffConfigTrayItem, false);
    config.value = await invoke<Config>("get_config");
    monitors.value = await invoke("get_monitors");

    //document.addEventListener('contextmenu', event => event.preventDefault());
});

watch(() => [
    config.value?.text_color,
    config.value?.text_align,
    config.value?.background_color,
    config.value?.interval,
    config.value?.text_size
], (_curr, prev) => {
    if (prev.every(v => v === undefined)) return;
    canSave.value = true;
});

watch(() => config.value?.blur_background, (_curr, prev) => {
    if (prev === undefined) return;
    saveConfig();
});

const saveConfig = async (rOverlay: boolean = true) => {
    try {
        await invoke<void>("set_config", {newConfig: config.value, refreshWOverlay: rOverlay});
        canSave.value = false;
    } catch (e) {
        console.error(e);
    }
};

const changeMonitor = (monitor: string) => {
    if (!config.value) return;
    config.value.monitor = parseInt(monitor);
    saveConfig();
};

const changeLang = (lang: string) => {
    if (!config.value) return;
    config.value.lang = lang;
    saveConfig();
};

const onSelect = async () => {
    try {
        await invoke("select_region", {monitor: config.value?.monitor || monitors.value[0].name || ''});
        await currWindow.close();
    } catch (e) {
        console.error(e);
    }
};

const onClose = async () => {
    if (config.value?.region) {
        await emit(Events.OnOffConfigTrayItem, true);
        await currWindow.close();
    } else {
        await exit(0);
    }
};
</script>

<template>
    <main v-if="config" ref="main">
        <h1>Configuration</h1>

        <div class="lang">
            <h2>Target language</h2>
            <CustomSelect :default-item="config.lang" :items="languages" @item-change="changeLang"/>
        </div>

        <div class="screen">
            <h2>Monitor</h2>
            <CustomSelect
                v-if="monitors.length > 0"
                :default-item="(config.monitor || monitors[0].id || 0).toString()"
                :items="monitors.map((m) => ({value: m.id.toString(), label: m.name ||'Monitor unnamed' }))"
                @item-change="changeMonitor"
            />
        </div>

        <div class="region-select">
            <div class="head">
                <h2>Screen region</h2>
                <CustomButton :is-primary="false" title="Select region" @click="onSelect">
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

        <div class="text-size">
            <h2>Text size</h2>
            <div>
                <CustomInput
                    v-model="config.text_size"
                    :max="250"
                    type="number"
                />
                px
            </div>
        </div>

        <div class="text-align">
            <h2>Text align</h2>
            <div class="grid">
                <span :class="{active: config.text_align === 'T:L'}" @click="config.text_align = 'T:L'">T:L</span>
                <span :class="{active: config.text_align === 'T:C'}" @click="config.text_align = 'T:C'">T:C</span>
                <span :class="{active: config.text_align === 'T:R'}" @click="config.text_align = 'T:R'">T:R</span>
                <span :class="{active: config.text_align === 'C:L'}" @click="config.text_align = 'C:L'">C:L</span>
                <span :class="{active: config.text_align === 'C:C'}" @click="config.text_align = 'C:C'">C:C</span>
                <span :class="{active: config.text_align === 'C:R'}" @click="config.text_align = 'C:R'">C:R</span>
                <span :class="{active: config.text_align === 'B:L'}" @click="config.text_align = 'B:L'">B:L</span>
                <span :class="{active: config.text_align === 'B:C'}" @click="config.text_align = 'B:C'">B:C</span>
                <span :class="{active: config.text_align === 'B:R'}" @click="config.text_align = 'B:R'">B:R</span>
            </div>
        </div>

        <div class="window-blur">
            <h2>Background blur (Need restart)</h2>
            <input id="blur" v-model="config.blur_background" name="blur" type="checkbox">
        </div>

        <div class="bg-color">
            <h2>Background color</h2>
            <ColorPicker
                v-model:pure-color="config.background_color"
                :z-index="20"
                format="rgb"
                lang="En"
                picker-type="chrome"
                theme="black"
            />
        </div>

        <div class="interval">
            <h2>Capture interval</h2>
            <div>
                <CustomInput
                    v-model="config.interval"
                    :max="60"
                    type="number"
                />
                s
            </div>
        </div>
    </main>

    <div class="action">
        <CustomButton :disabled="!canSave" :is-primary="true" title="Close" @click="saveConfig(false)">
            Save
        </CustomButton>
        <CustomButton :is-primary="true" title="Close" @click="onClose">
            {{ config?.region ? "Close" : "Quit" }}
        </CustomButton>
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

.region-select .head, .screen, .text-color, .text-align, .text-size, .window-blur, .bg-color, .interval, .lang {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.screen, .text-color, .text-align, .text-size, .window-blur, .bg-color, .interval, .region-select, .lang {
    background: #191919;
    padding: 10px;
    border-radius: 10px;
}

.region-select .info {
    display: flex;
    justify-content: space-between;
}

.grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
}

.grid span {
    border: 1px solid rgb(174, 174, 174);
    padding: 3px;
    font-size: 0.8em;
    border-radius: 3px;
    text-align: center;
}

.grid span:hover {
    background: rgba(255, 255, 255, 0.1);
}

.grid span.active {
    background: rgb(232, 232, 232);
    border-color: rgb(232, 232, 232);
    color: black;
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
