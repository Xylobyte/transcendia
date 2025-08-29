<!-- Copyright © 2025 Xylobyte
     SPDX-License-Identifier: AGPL-3.0-or-later -->

<script lang="ts" setup>
import {onMounted, ref} from "vue";
import {invoke} from "@tauri-apps/api/core";
import {Config, Region} from "../types/config.ts";
import {getCurrentWebviewWindow} from "@tauri-apps/api/webviewWindow";

const config = ref<Config>();

onMounted(async () => {
    config.value = await invoke<Config>("get_config");

    document.addEventListener('contextmenu', event => event.preventDefault());
});

const drawRegion = ref(false);
const startPos = ref([0, 0]);
const endPos = ref([0, 0]);

const startDraw = (evt: MouseEvent) => {
    drawRegion.value = true;
    startPos.value = [evt.clientX, evt.clientY];
};

const stopDraw = async () => {
    drawRegion.value = false;

    const width = endPos.value[0] - startPos.value[0];
    const height = endPos.value[1] - startPos.value[1];
    if (width > 50 && height > 50) {
        try {
            await invoke<void>("set_config", {
                newConfig: {
                    ...config.value,
                    region: {
                        x: startPos.value[0],
                        y: startPos.value[1],
                        w: width,
                        h: height,
                    } as Region
                } as Config,
                refreshWOverlay: true,
            });
            await invoke<void>("finish_select_region");
            await getCurrentWebviewWindow().close();
        } catch (e) {
            console.error(e);
        }
    }

    startPos.value = [0, 0];
    endPos.value = [0, 0];
};

const drawUpdate = (evt: MouseEvent) => {
    if (drawRegion.value) {
        endPos.value = [evt.clientX, evt.clientY];
    }
};
</script>

<template>
    <main @mousedown="startDraw" @mousemove="drawUpdate" @mouseup="stopDraw">
        <div class="info">
            <span>Select a capture region on the screen</span>
            <span>(Ctrl + X to cancel)</span>
        </div>

        <div
            v-if="drawRegion"
            :style="{left: `${startPos[0]}px`, top: `${startPos[1]}px`, width: `${endPos[0]-startPos[0]}px`, height: `${endPos[1]-startPos[1]}px`}"
            class="rect"
        ></div>
    </main>
</template>

<style scoped>
main {
    width: 100%;
    height: 100%;
    cursor: crosshair !important;
}

main * {
    cursor: inherit !important;
}

.info {
    display: flex;
    flex-direction: column;
    gap: 15px;
    justify-content: center;
    align-items: center;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: -1;
    opacity: 0.5;
}

.info span {
    color: white;
    text-shadow: black 0 0 10px;
    font-weight: bold;
    font-size: 2vw;
    user-select: none;
    -webkit-user-select: none;
}

.info span:last-child {
    font-size: 1.5em;
}

.rect {
    border: 2px solid white;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.5);
    box-shadow: 0 0 10px 1px rgba(0, 0, 0, 0.2);
    position: fixed;
}
</style>

<style>
html {
    background: transparent;
}

body {
    background: transparent;
}
</style>
