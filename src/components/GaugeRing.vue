<script setup lang="ts">
// Ported from UsageGaugeRing.swift + UsageWindow.swift: a deterministic SVG ring (drawn with
// stroke-dasharray/dashoffset, not an animated <progress>, so a snapshot never flakes) with the
// same fraction/percent/severity math UsageWindowTests.swift exercises.
import { computed } from 'vue';

// `size` is new in M2-21: this component's first real caller (ConversationContextCard.vue) needs
// 50px, not the 36px every prior analytics call site used (Swift's own `UsageGaugeRing` has no
// fixed size either -- each caller sets its own `.frame`). Optional, defaulting to 36, so nothing
// about the existing markup/assertions changes for a caller that doesn't pass it.
const props = defineProps<{ utilization: number; size?: number }>();
const size = computed(() => props.size ?? 36);

const WARNING_THRESHOLD = 80;
const CRITICAL_THRESHOLD = 95;

type Severity = 'normal' | 'warning' | 'critical';

function severityOf(utilization: number): Severity {
  if (utilization >= CRITICAL_THRESHOLD) return 'critical';
  if (utilization >= WARNING_THRESHOLD) return 'warning';
  return 'normal';
}

const fraction = computed(() => Math.min(Math.max(props.utilization / 100, 0), 1));
const percent = computed(() => Math.round(props.utilization));
const severity = computed(() => severityOf(props.utilization));

const tintVar = computed(() => ({ normal: '--color-green', warning: '--color-orange', critical: '--color-red' })[severity.value]);

const RADIUS = 15;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
const dashOffset = computed(() => CIRCUMFERENCE * (1 - fraction.value));
</script>

<template>
  <svg
    viewBox="0 0 36 36"
    :width="size"
    :height="size"
    role="img"
    :aria-label="`${percent}% used`"
  >
    <circle
      cx="18"
      cy="18"
      :r="RADIUS"
      fill="none"
      stroke="var(--bg-elevated)"
      stroke-width="5"
    />
    <circle
      cx="18"
      cy="18"
      :r="RADIUS"
      fill="none"
      :stroke="`var(${tintVar})`"
      stroke-width="5"
      stroke-linecap="round"
      :stroke-dasharray="CIRCUMFERENCE"
      :stroke-dashoffset="dashOffset"
      transform="rotate(-90 18 18)"
    />
    <text
      x="18"
      y="18"
      text-anchor="middle"
      dominant-baseline="central"
      fill="var(--text-primary)"
      font-size="10"
    >{{ percent }}%</text>
  </svg>
</template>
