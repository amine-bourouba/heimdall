<script setup lang="ts">
// Ported from ChartCard.swift:1-36 (docs/spec/analytics.md §7's own shared chrome every one of
// the 6 charts uses): title + optional subtitle header, content, padded, bgCard fill + border.
// The chart itself is a plain default slot -- AnalyticsDashboard.vue builds each chart's own
// ECharts `option` object and passes a <VChart> (or the heatmap grid) as this card's content, so
// this component stays generic/reusable rather than knowing about any one chart's data shape.
withDefaults(defineProps<{ title: string; subtitle?: string | null; height?: number }>(), { subtitle: null, height: 220 });
</script>

<template>
  <div class="flex flex-col gap-(--spacing-sm) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
    <div class="flex flex-col gap-(--spacing-xxs)">
      <h3 class="font-semibold text-(--text-primary) text-(length:--text-body-medium)">
        {{ title }}
      </h3>
      <p
        v-if="subtitle"
        class="text-(--text-tertiary) text-(length:--text-tiny)"
      >
        {{ subtitle }}
      </p>
    </div>
    <div :style="{ height: `${height}px` }">
      <slot />
    </div>
  </div>
</template>
