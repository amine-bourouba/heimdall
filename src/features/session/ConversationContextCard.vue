<script setup lang="ts">
// Ported from ConversationContextCard.swift + ContextUsage.swift's own `fraction`/`cacheHitRate`
// computed properties (neither is in the IPC contract -- `ContextUsage` carries only the raw
// fields `Message.from` already reduces: current/peak/window/*Tokens/series -- so both formulas
// are ported here, verbatim, as the one place that needs them). `GaugeRing.vue` (M2-07) already
// has the exact severity/ring math `UsageGaugeRing`+`UsageWindow` provided; this is its first
// real caller, at 50px instead of every prior analytics call site's 36px (GaugeRing.vue's own
// `size` prop, added for exactly this).
import { computed } from 'vue';
import GaugeRing from '@/components/GaugeRing.vue';
import { formatLargeNumber } from './format';
import type { ContextUsage } from '@/ipc/bindings';

const props = defineProps<{ context: ContextUsage }>();

// ContextUsage.swift:27-29,34-37, ported verbatim (not in the IPC contract).
const fraction = computed(() => (props.context.window > 0 ? Math.min(Math.max(props.context.current / props.context.window, 0), 1) : 0));
const cacheHitRate = computed(() => {
  const promptTotal = props.context.inputTokens + props.context.cacheReadTokens + props.context.cacheCreationTokens;
  return promptTotal > 0 ? props.context.cacheReadTokens / promptTotal : 0;
});

const cacheTokens = computed(() => props.context.cacheReadTokens + props.context.cacheCreationTokens);
const tokenMixTotal = computed(() => Math.max(1, props.context.inputTokens + props.context.outputTokens + cacheTokens.value));

function segmentWidth(value: number): string {
  return `${(value / tokenMixTotal.value) * 100}%`;
}

// ContextSparkline.path(in:), ported verbatim: a normalized polyline, heights relative to the
// window (so the line doubles as a fill indicator). Inline SVG, not a canvas/library chart --
// this task's own AC.
const SPARKLINE_WIDTH = 150;
const SPARKLINE_HEIGHT = 32;

const sparklinePoints = computed(() => {
  const values = props.context.series;
  if (values.length < 2 || props.context.window <= 0) return '';
  const stepX = SPARKLINE_WIDTH / (values.length - 1);
  return values
    .map((v, i) => {
      const f = Math.min(Math.max(v / props.context.window, 0), 1);
      const y = SPARKLINE_HEIGHT - f * SPARKLINE_HEIGHT;
      return `${i * stepX},${y}`;
    })
    .join(' ');
});

// UsageWindow's own severity tint, mirrored from GaugeRing.vue's identical thresholds (80/95) --
// kept local rather than exported from GaugeRing, since a <polyline> stroke and an SVG ring fill
// are different enough consumers that sharing one function would mean exporting plumbing whose
// only other job is deciding a CSS var name.
const sparklineColor = computed(() => {
  const percent = fraction.value * 100;
  if (percent >= 95) return 'var(--color-red)';
  if (percent >= 80) return 'var(--color-orange)';
  return 'var(--color-green)';
});
</script>

<template>
  <div class="flex items-center gap-(--spacing-lg) rounded-md border border-(--border) bg-(--bg-card) p-(--spacing-md)">
    <div class="flex items-center gap-(--spacing-md)">
      <GaugeRing
        :utilization="fraction * 100"
        :size="50"
      />
      <div class="flex flex-col gap-(--spacing-xxs)">
        <span class="text-(length:--text-small) text-(--text-secondary)">Context window</span>
        <span class="text-(length:--text-small) font-medium text-(--text-primary)">{{ formatLargeNumber(context.current) }} / {{ formatLargeNumber(context.window) }}</span>
        <span class="text-(length:--text-micro) text-(--text-tertiary)">{{ Math.round(cacheHitRate * 100) }}% from cache</span>
      </div>
    </div>

    <div class="h-11 w-px shrink-0 bg-(--border)/40" />

    <div class="flex flex-col gap-(--spacing-xs)">
      <span class="text-(length:--text-small) text-(--text-secondary)">Tokens</span>
      <div class="flex h-1.5 w-[150px] overflow-hidden rounded-full">
        <div
          class="h-full bg-(--color-blue)"
          :style="{ width: segmentWidth(cacheTokens) }"
        />
        <div
          class="h-full bg-(--color-teal)"
          :style="{ width: segmentWidth(context.inputTokens) }"
        />
        <div
          class="h-full bg-(--accent)"
          :style="{ width: segmentWidth(context.outputTokens) }"
        />
      </div>
      <div class="flex gap-(--spacing-sm)">
        <span class="flex items-center gap-(--spacing-xxs) text-(length:--text-micro) text-(--text-tertiary)">
          <span class="h-1.5 w-1.5 rounded-full bg-(--color-blue)" />Cache {{ formatLargeNumber(cacheTokens) }}
        </span>
        <span class="flex items-center gap-(--spacing-xxs) text-(length:--text-micro) text-(--text-tertiary)">
          <span class="h-1.5 w-1.5 rounded-full bg-(--color-teal)" />In {{ formatLargeNumber(context.inputTokens) }}
        </span>
        <span class="flex items-center gap-(--spacing-xxs) text-(length:--text-micro) text-(--text-tertiary)">
          <span class="h-1.5 w-1.5 rounded-full bg-(--accent)" />Out {{ formatLargeNumber(context.outputTokens) }}
        </span>
      </div>
    </div>

    <div class="flex-1" />

    <div
      v-if="context.series.length > 1"
      class="flex flex-col items-end gap-(--spacing-xxs)"
    >
      <span class="text-(length:--text-micro) text-(--text-tertiary)">Context over {{ context.series.length }} turns</span>
      <svg
        role="img"
        aria-label="Context size per turn"
        :width="SPARKLINE_WIDTH"
        :height="SPARKLINE_HEIGHT"
        :viewBox="`0 0 ${SPARKLINE_WIDTH} ${SPARKLINE_HEIGHT}`"
      >
        <polyline
          :points="sparklinePoints"
          fill="none"
          :stroke="sparklineColor"
          stroke-width="1.5"
          stroke-linejoin="round"
        />
      </svg>
    </div>
  </div>
</template>
