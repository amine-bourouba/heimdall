<script setup lang="ts">
// Ported from ContributionHeatmap.swift (M3-34; docs/spec/analytics.md §6). Swift's own grid is
// week-major: an HStack of weeks, each a VStack of 7 day-of-week cells (`grid[week][day]`,
// ContributionHeatmap.swift:16-43) -- kept identical here, column = week, row = day-of-week
// within that week-column, Monday first.
//
// ARIA: the outer grid's `role="row"` groups are the week-columns (matching the DOM nesting
// above, not a literal horizontal pixel-row) -- each is a valid ARIA row of 7 `gridcell`s
// regardless of which calendar axis it visually represents, so a screen-reader user arrows
// through one week's 7 days, then moves to the next week. Pure layout padding cells (days before
// the first real entry or after the last) carry no date and are `aria-hidden` rather than empty
// `gridcell`s, so AT navigation doesn't stop on "empty, empty, empty...".
import { computed } from 'vue';
import { useAnalyticsStore } from '../store';
import { localDateKey } from '../charts/formatters';
import ChartCard from './ChartCard.vue';

const store = useAnalyticsStore();

interface HeatmapCell {
  dateString: string | null;
  messages: number;
  sessions: number;
}

const heatmapGrid = computed<HeatmapCell[][]>(() => {
  const entries = store.heatmapData;
  if (entries.length === 0) return [];
  const sorted = [...entries].sort((a, b) => a.date.getTime() - b.date.getTime());
  const first = sorted[0].date;
  const last = sorted[sorted.length - 1].date;
  const firstWeekday = first.getDay(); // 0=Sun..6=Sat
  const totalDays = Math.round((last.getTime() - first.getTime()) / 86_400_000);
  const adjustedFirstDay = (firstWeekday + 6) % 7; // remap to Monday=0..Sunday=6
  const weekCount = Math.ceil((adjustedFirstDay + totalDays + 1) / 7);
  const grid: HeatmapCell[][] = Array.from({ length: weekCount }, () => Array.from({ length: 7 }, () => ({ dateString: null, messages: 0, sessions: 0 })));
  const byDateString = new Map(entries.map((e) => [e.dateString, e]));
  for (let offset = 0; offset <= totalDays; offset++) {
    const date = new Date(first);
    date.setDate(date.getDate() + offset);
    const weekday = date.getDay();
    const row = (weekday + 6) % 7;
    const col = Math.floor((adjustedFirstDay + offset) / 7);
    const dateString = localDateKey(date);
    const entry = byDateString.get(dateString);
    grid[col][row] = { dateString, messages: entry?.messages ?? 0, sessions: entry?.sessions ?? 0 };
  }
  return grid;
});
const heatmapMax = computed(() => Math.max(1, ...store.heatmapData.map((e) => e.messages)));
function heatmapCellClass(cell: HeatmapCell): string {
  if (!cell.dateString || cell.messages === 0) return 'bg-(--bg-card)';
  const ratio = cell.messages / heatmapMax.value;
  if (ratio < 0.25) return 'bg-(--accent)/20';
  if (ratio < 0.5) return 'bg-(--accent)/40';
  if (ratio < 0.75) return 'bg-(--accent)/65';
  return 'bg-(--accent)';
}
function cellAriaLabel(cell: HeatmapCell): string | undefined {
  return cell.dateString ? `${cell.dateString}: ${cell.messages} messages, ${cell.sessions} sessions` : undefined;
}
const DAY_LABELS = ['Mon', '', 'Wed', '', 'Fri', '', 'Sun'];
</script>

<template>
  <ChartCard
    title="Activity Heatmap"
    subtitle="Daily message intensity"
    :height="heatmapGrid.length ? 140 : 60"
  >
    <div
      v-if="heatmapGrid.length"
      class="flex h-full gap-(--spacing-xxs)"
    >
      <div class="flex flex-col justify-between py-0.5 text-(--text-tertiary) text-(length:--text-pico)">
        <span
          v-for="(label, i) in DAY_LABELS"
          :key="i"
          class="h-3"
        >{{ label }}</span>
      </div>
      <div
        role="grid"
        aria-label="Activity heatmap"
        class="flex gap-0.5 overflow-x-auto"
      >
        <div
          v-for="(week, wi) in heatmapGrid"
          :key="wi"
          role="row"
          class="flex flex-col gap-0.5"
        >
          <div
            v-for="(cell, di) in week"
            :key="di"
            :role="cell.dateString ? 'gridcell' : undefined"
            :aria-hidden="cell.dateString ? undefined : 'true'"
            :aria-label="cellAriaLabel(cell)"
            :title="cell.dateString ? `${cell.dateString}: ${cell.messages} messages · ${cell.sessions} sessions` : undefined"
            class="h-3 w-3 rounded-xs"
            :class="heatmapCellClass(cell)"
          />
        </div>
      </div>
    </div>
  </ChartCard>
</template>
