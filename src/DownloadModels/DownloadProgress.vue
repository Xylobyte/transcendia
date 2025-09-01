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
import {emit, listen, UnlistenFn} from "@tauri-apps/api/event";
import {onMounted, onUnmounted, ref, watch} from "vue";
import {Events} from "../types/events.ts";
import {DownloadProgress} from "../types/download.ts";
import Progress from "../components/Progress.vue";
import CustomButton from "../components/CustomButton.vue";
import {invoke} from "@tauri-apps/api/core";
import {getCurrentWebviewWindow} from "@tauri-apps/api/webviewWindow";

const currWindow = getCurrentWebviewWindow();

const progressDetectionFile = ref<DownloadProgress>();
const progressRecognitionFile = ref<DownloadProgress>();

let unlisten: UnlistenFn;
onMounted(async () => {
    unlisten = await listen<DownloadProgress>(Events.DownloadProgress, (event) => {
        if (event.payload.file.includes("detection")) {
            progressDetectionFile.value = event.payload;
        } else if (event.payload.file.includes("recognition")) {
            progressRecognitionFile.value = event.payload;
        }
    });
});

onUnmounted(() => {
    unlisten();
});

watch([progressDetectionFile, progressRecognitionFile], async () => {
    if (
        progressDetectionFile.value?.progress === progressDetectionFile.value?.total_size &&
        progressRecognitionFile.value?.progress === progressRecognitionFile.value?.total_size
    ) {
        await invoke("download_finish");
        await currWindow.close();
    }
});

const handleQuit = async () => {
    await emit(Events.StopDownload);
};
</script>

<template>
    <main>
        <h1>Download models for ocr...</h1>

        <div v-if="progressDetectionFile">
            <span>{{ progressDetectionFile.file }}</span>
            <Progress :progress="progressDetectionFile.progress" :total="progressDetectionFile.total_size"/>
        </div>
        <div v-if="progressRecognitionFile">
            <span>{{ progressRecognitionFile.file }}</span>
            <Progress :progress="progressRecognitionFile.progress" :total="progressRecognitionFile.total_size"/>
        </div>

        <CustomButton :is-primary="false" title="Stop and quit" @click="handleQuit">
            Stop and quit
        </CustomButton>
    </main>
</template>

<style scoped>
main {
    display: flex;
    flex-direction: column;
    padding: 20px;
    gap: 30px;
    width: 100%;
    height: 100%;

    align-items: center;

    > div {
        display: flex;
        flex-direction: column;
        gap: 6px;
        width: 100%;

        span {
            font-size: 0.8em;
        }
    }
}

h1 {
    margin: 0;
    font-size: 1.6rem;
}
</style>

<style>
body {
    background: var(--background);
}
</style>
