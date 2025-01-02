<script lang="ts" setup>
import {computed, CSSProperties, onMounted, onUnmounted, ref} from "vue";
import {Config} from "../types/config.ts";
import {invoke} from "@tauri-apps/api/core";
import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {Events} from "../types/events.ts";

const config = ref<Config>();

let unlisten: UnlistenFn;

onMounted(async () => {
    await getConfig();

    unlisten = await listen(Events.RefreshOverlay, () => {
        getConfig();
    });
});

onUnmounted(() => {
    unlisten();
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
        <p :style="mainStyle">
            Ipsum aliqua eiusmod qui cillum veniam minim tempor culpa consectetur anim deserunt reprehenderit laboris
            cupidatat. Mollit excepteur dolor anim anim labore in in tempor minim. Dolor anim cupidatat labore do sit.
            Amet incididunt qui cillum magna nisi velit quis amet elit veniam aliquip aliquip fugiat sint. Anim quis et
            pariatur ipsum esse laborum ipsum mollit dolore consequat fugiat non.
        </p>
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
}
</style>