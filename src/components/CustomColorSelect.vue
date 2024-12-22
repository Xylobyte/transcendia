<script lang="ts" setup>
const model = defineModel<string>();
</script>

<template>
    <input v-model="model" type="color">
</template>

<style scoped>
input {
    outline: none;
    border: 1px solid white;
    border-radius: 6px;
    background-color: transparent;
}
</style>
