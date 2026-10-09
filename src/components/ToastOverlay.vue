<script setup lang="ts">
// Ported from ToastOverlay.swift: renders the current toast from useToastsStore (M2-05), which
// already owns the single-item-queue and auto-dismiss timing this view just has to display.
import { computed } from 'vue';
import AppIcon from './AppIcon.vue';
import { useToastsStore } from '@/stores/toasts';

const toastsStore = useToastsStore();

const iconByStyle = { success: 'checkmark.circle.fill', error: 'xmark.circle.fill', info: 'info.circle' } as const;
const colorVarByStyle = { success: '--color-green', error: '--color-red', info: '--color-blue' } as const;

const icon = computed(() => (toastsStore.current ? iconByStyle[toastsStore.current.style] : null));
const colorVar = computed(() => (toastsStore.current ? colorVarByStyle[toastsStore.current.style] : null));
</script>

<template>
  <div
    class="pointer-events-none fixed inset-x-0 top-(--spacing-md) flex justify-center"
    aria-live="polite"
  >
    <button
      v-if="toastsStore.current"
      type="button"
      class="glass pointer-events-auto flex items-center gap-(--spacing-sm) rounded-full px-(--spacing-lg) py-(--spacing-md)"
      @click="toastsStore.clickCurrent()"
    >
      <AppIcon
        :name="icon!"
        :style="{ color: `var(${colorVar})` }"
      />
      <span class="text-(--text-primary) text-center">{{ toastsStore.current.message }}</span>
    </button>
  </div>
</template>
