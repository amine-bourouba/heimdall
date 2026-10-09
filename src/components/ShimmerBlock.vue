<script setup lang="ts">
// Ported from ShimmerModifier.swift's `shimmer(cornerRadius:)`: a loading-skeleton block with a
// gradient sweep. Gated the same way every other motion effect in this codebase is (M2-04):
// reduced motion or showAnimations off means a plain static block, not a toned-down shimmer.
import { computed } from 'vue';
import { usePrefersReducedMotion } from '@/composables/usePrefersReducedMotion';
import { usePrefsStore } from '@/stores/prefs';

withDefaults(defineProps<{ rounded?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' }>(), { rounded: 'md' });

const { prefersReducedMotion } = usePrefersReducedMotion();
const prefsStore = usePrefsStore();
const shouldShimmer = computed(() => !prefersReducedMotion.value && prefsStore.showAnimations);
</script>

<template>
  <div
    class="relative overflow-hidden bg-(--bg-card-hover)"
    :class="`rounded-${rounded}`"
  >
    <div
      v-if="shouldShimmer"
      class="shimmer-sweep"
    />
  </div>
</template>

<style scoped>
.shimmer-sweep {
  position: absolute;
  inset: 0;
  background: linear-gradient(to right, transparent, color-mix(in srgb, white 10%, transparent), transparent);
  animation: shimmer-sweep 1.5s linear infinite;
}

@keyframes shimmer-sweep {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(100%);
  }
}
</style>
