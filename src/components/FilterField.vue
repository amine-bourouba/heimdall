<script setup lang="ts">
// Ported from ConfigFilterField.swift, plus Esc-to-clear (this task's own AC; Poirot's version
// only clears via the button).
import { ref } from 'vue';
import AppIcon from './AppIcon.vue';

const modelValue = defineModel<string>({ default: '' });
withDefaults(defineProps<{ placeholder?: string }>(), { placeholder: 'Filter…' });

const inputRef = ref<HTMLInputElement>();

function clear() {
  modelValue.value = '';
  inputRef.value?.focus();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && modelValue.value !== '') {
    event.stopPropagation(); // clear the filter first; a second Esc can still close a parent sheet
    clear();
  }
}
</script>

<template>
  <div class="flex items-center gap-(--spacing-sm) rounded-md px-(--spacing-md) py-(--spacing-sm)">
    <AppIcon
      name="magnifyingglass"
      class="text-(--text-tertiary)"
    />
    <input
      ref="inputRef"
      v-model="modelValue"
      type="text"
      :placeholder="placeholder"
      :aria-label="placeholder"
      class="min-w-0 flex-1 bg-transparent text-(--text-primary) outline-none"
      @keydown="onKeydown"
    >
    <button
      v-if="modelValue !== ''"
      type="button"
      aria-label="Clear filter"
      title="Clear filter"
      @click="clear"
    >
      <AppIcon
        name="xmark.circle.fill"
        class="text-(--text-tertiary)"
      />
    </button>
  </div>
</template>
