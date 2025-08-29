<!-- Copyright © 2025 Xylobyte
     SPDX-License-Identifier: AGPL-3.0-or-later -->

<script lang="ts" setup>
defineProps<{
    type?: "text" | "number";
    placeholder?: string;
    max?: number;
}>();

const model = defineModel();
</script>

<template>
    <input v-model="model" :max="$props.max" :placeholder="$props.placeholder" :type="$props.type">
</template>

<style scoped>
input {
    background: transparent;
    border: 1px solid white;
    border-radius: 6px;
    padding: 6px;
    width: 60px;
    text-align: center;
}
</style>