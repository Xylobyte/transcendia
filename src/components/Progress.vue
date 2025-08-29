<!-- Copyright © 2025 Xylobyte
     SPDX-License-Identifier: AGPL-3.0-or-later -->

<script lang="ts" setup>
defineProps<{
    progress: number,
    total: number,
}>();
</script>

<template>
    <div class="progress">
        <div :style="{width: `${$props.progress / $props.total * 100}%`}"></div>
    </div>
</template>

<style scoped>
.progress {
    border-radius: 5px;
    border: 1px solid #d6d6d6;
    background: transparent;
    overflow: hidden;
    width: 100%;
    height: 20px;

    div {
        height: 100%;
        width: 50px;
        background: var(--primary);
    }
}
</style>