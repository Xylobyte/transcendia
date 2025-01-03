<script lang="ts" setup>
defineProps<{
    type?: "text" | "number";
    placeholder?: string;
}>();

const model = defineModel();
</script>

<template>
    <input v-model="model" :placeholder="$props.placeholder" :type="$props.type">
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