<script setup lang="ts">
// Ported from InfoTooltipButton.swift: click-to-open (not hover) is deliberate there too --
// "a plain .help tooltip on a non-interactive image is easy to miss and never fires on tap."
import { ref } from 'vue';

defineProps<{ text: string }>();

const isOpen = ref(false);
const buttonRef = ref<HTMLButtonElement>();

function toggle() {
  isOpen.value = !isOpen.value;
}

function close() {
  isOpen.value = false;
  buttonRef.value?.focus();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    event.stopPropagation();
    close();
  }
}
</script>

<template>
  <span
    class="relative inline-block"
    @keydown="onKeydown"
  >
    <button
      ref="buttonRef"
      type="button"
      aria-label="More info"
      aria-haspopup="true"
      :aria-expanded="isOpen"
      title="More info"
      @click="toggle"
    >
      <svg
        viewBox="0 0 12 12"
        width="12"
        height="12"
        class="text-(--text-tertiary)"
      ><circle
        cx="6"
        cy="6"
        r="5.5"
        fill="none"
        stroke="currentColor"
      /><text
        x="6"
        y="6"
        text-anchor="middle"
        dominant-baseline="central"
        font-size="8"
        fill="currentColor"
      >i</text></svg>
    </button>
    <div
      v-if="isOpen"
      role="tooltip"
      class="glass absolute top-full left-1/2 z-10 mt-(--spacing-xs) w-65 -translate-x-1/2 rounded-md p-(--spacing-md) text-(--text-secondary) text-(length:--text-small)"
    >
      {{ text }}
    </div>
  </span>
</template>
