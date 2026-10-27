<script setup lang="ts">
// Ported from StatCard.swift:1-57 (docs/spec/analytics.md §8). Pre-formatted props only
// (`value`/`subtitle` are already display strings) -- AnalyticsDashboard.vue owns every formatter
// call (formatLargeNumber/formatCost/etc.), keeping this component a plain presentational card,
// the same division of labour every other dumb/display component in this port already uses.
import InfoTooltipButton from '@/components/InfoTooltipButton.vue';
import AppIcon from '@/components/AppIcon.vue';

withDefaults(
  defineProps<{
    title: string;
    value: string;
    subtitle?: string | null;
    icon: string;
    color: 'accent' | 'blue' | 'purple' | 'teal' | 'green' | 'orange';
    dimmed?: boolean;
    infoTooltip?: string | null;
  }>(),
  { subtitle: null, dimmed: false, infoTooltip: null },
);

const COLOR_CLASS: Record<string, string> = {
  accent: 'text-(--accent)',
  blue: 'text-(--color-blue)',
  purple: 'text-(--color-purple)',
  teal: 'text-(--color-teal)',
  green: 'text-(--color-green)',
  orange: 'text-(--color-orange)',
};
</script>

<template>
  <div class="flex h-27.5 flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-md)">
    <div class="flex items-start justify-between">
      <AppIcon
        :name="icon"
        size="sm"
        :class="COLOR_CLASS[color]"
      />
      <InfoTooltipButton
        v-if="infoTooltip"
        :text="infoTooltip"
      />
    </div>
    <p
      class="text-(length:--text-heading-sm) font-semibold"
      :class="dimmed ? 'text-(--text-tertiary)' : 'text-(--text-primary)'"
    >
      {{ value }}
    </p>
    <p class="text-(--text-secondary) text-(length:--text-small)">
      {{ title }}
    </p>
    <!-- Literal single space when subtitle is absent (StatCard.swift:35-41): holds the row height
         constant across the 8-card grid instead of some cards being one line shorter. -->
    <p class="text-(--text-tertiary) text-(length:--text-tiny)">
      {{ subtitle ?? ' ' }}
    </p>
  </div>
</template>
