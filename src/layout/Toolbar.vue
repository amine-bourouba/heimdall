<script setup lang="ts">
// Ported from ContentView.swift's `toolbarNavigation` (back/forward only -- `toolbarContextual`,
// the session/config-detail action buttons, is each owning feature's own job once it exists).
import { useNavigation } from '@/composables/useNavigation';
import AppIcon from '@/components/AppIcon.vue';

const { canGoBack, canGoForward, goBack, goForward } = useNavigation();
</script>

<template>
  <div
    role="toolbar"
    aria-label="Navigation"
    class="flex items-center gap-(--spacing-xs) px-(--spacing-md) py-(--spacing-sm)"
  >
    <button
      type="button"
      aria-label="Back"
      title="Back (⌘[)"
      :disabled="!canGoBack"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary) disabled:opacity-40"
      @click="goBack"
    >
      <AppIcon
        name="chevron.left"
        size="sm"
      />
    </button>
    <button
      type="button"
      aria-label="Forward"
      title="Forward (⌘])"
      :disabled="!canGoForward"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary) disabled:opacity-40"
      @click="goForward"
    >
      <AppIcon
        name="chevron.right"
        size="sm"
      />
    </button>
  </div>
</template>
