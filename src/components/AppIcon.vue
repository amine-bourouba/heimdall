<script setup lang="ts">
// The only sanctioned way to render a Lucide icon (AI_INSTRUCTIONS.md: "Use Lucide icons only
// through src/components/AppIcon.vue"). `effect` applies one of motion.css's named utilities,
// gated by both OS-level reduced motion and the showAnimations preference -- failing either
// gate means no animation class at all, not a toned-down one.
import { computed } from 'vue';
import { CircleQuestionMark } from '@lucide/vue';
import { usePrefersReducedMotion } from '@/composables/usePrefersReducedMotion';
import { usePrefsStore } from '@/stores/prefs';
import { iconMap } from './iconMap';

export type MotionEffect = 'bounce' | 'pulse' | 'breathe' | 'variable-color' | 'rotate' | 'wiggle' | 'replace';

const props = defineProps<{
  /** An SF Symbol name from docs/design/icon-map.md -- a name with no entry there falls back to
   * a visible "unmapped icon" marker rather than silently rendering nothing. */
  name: string;
  effect?: MotionEffect;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}>();

const { prefersReducedMotion } = usePrefersReducedMotion();
const prefsStore = usePrefsStore();

const icon = computed(() => iconMap[props.name] ?? CircleQuestionMark);

const motionClass = computed(() => {
  if (!props.effect) return undefined;
  if (prefersReducedMotion.value || !prefsStore.showAnimations) return undefined;
  return `motion-${props.effect}`;
});

const sizeStyle = computed(() => (props.size ? { width: `var(--icon-size-${props.size})`, height: `var(--icon-size-${props.size})` } : undefined));
</script>

<template>
  <component
    :is="icon"
    :class="motionClass"
    :style="sizeStyle"
  />
</template>
