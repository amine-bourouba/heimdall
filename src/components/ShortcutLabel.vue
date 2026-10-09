<script setup lang="ts">
// Renders a shortcut combo with the real platform-aware modifier label (usePlatformStore, M2-05):
// "⌘K" on macOS, "Ctrl+K" elsewhere -- never hand-picks ⌘ for every OS the way a hardcoded label
// would (parity row J: "On Windows and Linux, ⌘ becomes Ctrl everywhere, including labels").
import { computed } from 'vue';
import { usePlatformStore } from '@/stores/platform';

const props = defineProps<{ keyLabel: string }>();

const platformStore = usePlatformStore();
const isMac = computed(() => platformStore.os === 'macos');
const label = computed(() => (isMac.value ? `${platformStore.modifierLabel}${props.keyLabel}` : `${platformStore.modifierLabel}+${props.keyLabel}`));
</script>

<template>
  <kbd class="rounded-sm border px-(--spacing-xs) text-(length:--text-small) text-(--text-tertiary)">{{ label }}</kbd>
</template>
