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
import {availableMonitors, Monitor} from "@tauri-apps/api/window";
import CustomColorSelect from "../components/CustomColorSelect.vue";

const currWindow = getCurrentWebviewWindow();

const monitors = ref<Monitor[]>([]);
const config = ref<Config>();
const canSave = ref(false);

onMounted(async () => {
    await emit(Events.OnOffConfigTrayItem, false);
    config.value = await invoke<Config>("get_config");
    monitors.value = await availableMonitors();

    document.addEventListener('contextmenu', event => event.preventDefault());
});

watch(() => [config.value?.text_color, config.value?.text_align], (curr, prev) => {
    if (prev.every(v => !v)) return;
    canSave.value = true;
});

const saveConfig = async () => {
    try {
        await invoke<void>("set_config", {newConfig: config.value});
        canSave.value = false;
    } catch (e) {
        console.error(e);
    }
};

const changeMonitor = (monitor: string) => {
    if (!config.value) return;
    config.value.monitor = monitors.value.findIndex(m => m.name === monitor) || 0;
    saveConfig();
};

const onSelect = async () => {
    try {
        await invoke("select_region", {monitor: config.value?.monitor || 0});
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

        <div class="screen">
            <h2>Monitor</h2>
            <CustomSelect
                v-if="monitors.length > 0"
                :default-item="monitors[config.monitor || 0].name || '0'"
                :items="monitors.map((m, i) => ({value: m.name || i.toString(), label: `Screen ${i}`}))"
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
            <CustomColorSelect v-model="config.text_color"/>
        </div>

        <div class="text-color">
            <h2>Background color</h2>
            <CustomColorSelect v-model="config.background_color"/>
        </div>

        <div class="action">
            <CustomButton :disabled="!canSave" :is-primary="true" title="Close" @click="saveConfig">
                Save
            </CustomButton>
            <CustomButton :is-primary="true" title="Close" @click="onClose">
                {{ config.region ? "Close" : "Quit" }}
            </CustomButton>
        </div>
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
}

h1 {
    margin: 0;
    font-size: 2rem;
}

h2 {
    font-size: 1.3rem;
    font-weight: normal;
    color: rgb(232, 232, 232);
    margin: 0;
}

.region-select {
    display: flex;
    flex-direction: column;
    gap: 15px;
}

.region-select .head, .screen, .text-color {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.region-select .info {
    display: flex;
    justify-content: space-between;
}

.action {
    display: flex;
    gap: 10px;
    position: absolute;
    bottom: 20px;
    right: 20px;
}
</style>

<style>
body {
    background: var(--background);
}
</style>
