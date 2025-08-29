<!-- Copyright © 2025 Xylobyte
     SPDX-License-Identifier: AGPL-3.0-or-later -->

<script lang="ts" setup>
defineProps<{
    title: string
    isPrimary: boolean
    disabled?: boolean
}>();
</script>

<template>
    <button
        :class="{primary: $props.isPrimary}"
        :disabled="$props.disabled"
        :title="$props.title"
    >
        <slot/>
    </button>
</template>

<style scoped>
button {
    display: flex;
    align-items: center;
    outline: none;
    border-radius: 6px;
    background: transparent;
    font-size: 1rem;
    border: 1px solid #d6d6d6;
    color: white;
    padding: 6px 15px;
}

button:disabled {
    pointer-events: none;
    opacity: 0.5;
}

button:hover {
    background: rgba(255, 255, 255, 0.1);
}

button.primary {
    background: var(--primary);
    color: var(--on-primary);
    border: none;
}

button.primary:hover {
    opacity: 0.8;
}
</style>