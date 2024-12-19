<script lang="ts" setup>
defineProps<{
    items: { value: string, label: string }[];
    defaultItem: string;
}>();

defineEmits<{
    itemChange: [value: string];
}>();
</script>

<template>
    <div class="select">
        <select @change="e => $emit('itemChange', (e.target as HTMLSelectElement).value)">
            <option
                v-for="item in $props.items"
                :key="item.value"
                :selected="$props.defaultItem === item.value"
                :value="item.value"
            >
                {{ item.label }}
            </option>
        </select>
    </div>
</template>

<style scoped>
select {
    appearance: none;
    background-color: transparent;
    border: none;
    padding: 6px 15px;
    margin: 0;
    width: 100%;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    z-index: 1;
    outline: none;
}

.select {
    display: grid;
    grid-template-areas: "select";
    align-items: center;
    position: relative;
    border: 1px solid white;
    border-radius: 6px;
    min-width: 150px;
    font-size: 1rem;
    background-color: transparent;
}

.select:hover {
    background-color: rgba(255, 255, 255, 0.1);
}

select, .select::after {
    grid-area: select;
}

.select::after {
    content: "";
    margin-right: 1em;
    justify-self: end;
    width: 0.8em;
    height: 0.5em;
    background-color: white;
    clip-path: polygon(100% 0%, 0 0%, 50% 100%);
}
</style>