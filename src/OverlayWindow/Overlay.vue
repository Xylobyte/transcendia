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
        alignSelf: vAlign === "T" ? "start" : vAlign === "C" ? "center" : "end"
    } as CSSProperties;
});

const getConfig = async () => {
    config.value = await invoke<Config>("get_config");
};
</script>

<template>
    <main :style="{background: config?.background_color, height: config?.blur_background ? '100%' : 'fit-content'}">
        <p :style="mainStyle">{{ text }}</p>
    </main>
</template>

<style scoped>
main {
    border-radius: 30px;
    justify-content: center;
    display: flex;
    padding: 20px;
}

p {
    width: 100%;
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