<script setup lang="ts">
import {onMounted, onUnmounted, ref} from "vue";

const drawRegion = ref(false);
const startPos = ref([0, 0]);
const endPos = ref([0, 0]);
const main = ref<HTMLDivElement>();

onMounted(() => {
    if (!main.value) return;

    main.value.addEventListener("mousedown", startDraw);
    main.value.addEventListener("mouseup", stopDraw);
    main.value.addEventListener("mousemove", drawUpdate);
});

onUnmounted(() => {
    if (!main.value) return;

    main.value.removeEventListener("mousedown", startDraw);
    main.value.removeEventListener("mouseup", stopDraw);
});

const startDraw = (evt: MouseEvent) => {
    drawRegion.value = true;
    startPos.value = [evt.clientX, evt.clientY];
};

const stopDraw = () => {
    drawRegion.value = false;
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
    <main ref="main">
        <div class="info">
            <span>Select a capture region on the screen</span>
        </div>

        <div
            v-if="drawRegion"
            :style="{left: `${startPos[0]}px`, top: `${startPos[1]}px`, width: `${endPos[0]-startPos[0]}px`, height: `${endPos[1]-startPos[1]}px`}"
            class="rect"></div>
    </main>
</template>

<style>
html {
    background: transparent;
}

body {
    background: rgba(0, 0, 0, 0);
    width: 100vw;
    height: 100vh;
    margin: 0;
    padding: 0;
}

main {
    width: 100%;
    height: 100%;
    cursor: move;
}

.info {
    display: flex;
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
    font-family: sans-serif;
    user-select: none;
    -webkit-user-select: none;
}

.rect {
    border: 2px solid white;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.5);
    box-shadow: 0 0 10px 1px rgba(0, 0, 0, 0.2);
    position: fixed;
}
</style>
