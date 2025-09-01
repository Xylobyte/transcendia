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
import {computed, CSSProperties, onMounted, onUnmounted, ref} from "vue";
import {Config} from "../types/config.ts";
import {invoke} from "@tauri-apps/api/core";
import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {Events} from "../types/events.ts";

const config = ref<Config>();
const text = ref<string>("Loading...");

let unlistenRefresh: UnlistenFn;
let unlistenNewText: UnlistenFn;

onMounted(async () => {
    await getConfig();

    unlistenRefresh = await listen(Events.RefreshOverlay, () => {
        getConfig();
    });

    unlistenNewText = await listen(Events.NewTranslatedText, (event) => {
        text.value = event.payload as string;
    });
});

onUnmounted(() => {
    unlistenRefresh();
    unlistenNewText();
});

const mainStyle = computed(() => {
    const alignArray = config.value?.text_align.split(":") || ["C", "C"];
    const vAlign = alignArray[0];
    const hAlign = alignArray[1];
    return {
        color: config.value?.text_color,
        textAlign: hAlign === "L" ? "left" : hAlign === "C" ? "center" : "right",
        alignSelf: vAlign === "T" ? "start" : vAlign === "C" ? "center" : "end",
        fontSize: config.value?.text_size + 'px'
    } as CSSProperties;
});

const getConfig = async () => {
    config.value = await invoke<Config>("get_config");
};
</script>

<template>
    <main
        v-if="text"
        :style="{background: config?.background_color, height: config?.blur_background ? '100%' : 'fit-content', width: config?.blur_background ? '100%' : 'fit-content'}">
        <p :style="mainStyle">{{ text }}</p>
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