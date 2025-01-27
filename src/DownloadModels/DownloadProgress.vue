<script lang="ts" setup>
import {emit, listen, UnlistenFn} from "@tauri-apps/api/event";
import {onMounted, onUnmounted, ref, watch} from "vue";
import {Events} from "../types/events.ts";
import {DownloadProgress} from "../types/download.ts";
import Progress from "../components/Progress.vue";
import CustomButton from "../components/CustomButton.vue";
import {invoke} from "@tauri-apps/api/core";

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
