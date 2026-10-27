<script setup lang="ts">
// Split out of AnalyticsDashboard.vue (M3-32 necessary ripple, bundle-size fix, the same pattern
// M3-12 already established for Shiki): ECharts + vue-echarts alone pushed the analytics route
// to 197.64 KB gzip against the 150 KB lazy-route budget (port-plan.md §9), confirmed directly via
// `node scripts/check-bundle-size.mjs` against a real build -- not assumed. `AnalyticsDashboard.vue`
// now `defineAsyncComponent`s this file instead of eagerly importing ECharts itself, so the stat
// cards/CSV menu/loading/no-data states (which need no charting library at all) stay in the light,
// eager chunk, and the real chart weight only loads once the dashboard actually has stats to draw.
// No props: this calls the same singleton stores (`useAnalyticsStore`/`useThemeStore`) directly
// rather than threading everything through as props.
//
// Chart colours: a real, deliberate deviation from the Swift reference app, not a literal port.
// analytics.md §12.3 names the Swift behaviour as an open question: each of ModelUsageChart/
// CostBreakdownView/TokenUsageOverTimeChart independently re-slices the same 6-colour palette
// against its own differently-sorted model list (tokens desc / cost desc / alphabetical), so the
// *same model renders a different colour on different charts*. That is a real design smell, not a
// decision worth preserving -- `colorForModel` below assigns one stable colour per model name,
// computed once from the full sorted set of every model name appearing anywhere in the loaded
// stats, shared by every chart. This follows the dataviz skill's own non-negotiable: "Color
// follows the entity, never its rank. A filter that changes the series count must not repaint the
// survivors." A 7th+ model (beyond the fixed 6-colour palette, matching Swift's own no-wraparound
// cap) gets a neutral grey rather than silently reusing an earlier colour.
import { computed } from 'vue';
import { use } from 'echarts/core';
import { BarChart, LineChart, PieChart } from 'echarts/charts';
import { DatasetComponent, GridComponent, LegendComponent, TooltipComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import type { ComposeOption } from 'echarts/core';
import type { BarSeriesOption, LineSeriesOption, PieSeriesOption } from 'echarts/charts';
import type { DatasetComponentOption, GridComponentOption, LegendComponentOption, TooltipComponentOption } from 'echarts/components';
import { useThemeStore } from '@/stores/theme';
import { useAnalyticsStore, friendlyModelName } from '../store';
import AppIcon from '@/components/AppIcon.vue';
import ChartCard from './ChartCard.vue';

use([BarChart, LineChart, PieChart, DatasetComponent, GridComponent, LegendComponent, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<BarSeriesOption | LineSeriesOption | PieSeriesOption | GridComponentOption | LegendComponentOption | TooltipComponentOption | DatasetComponentOption>;

const store = useAnalyticsStore();
const themeStore = useThemeStore();

// --- Formatters duplicated from AnalyticsDashboard.vue (AnalyticsFormatters.swift, analytics.md
// §4) -- two small pure functions, not worth a shared module split across the lazy boundary for. ---
function formatLargeNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}
function formatCost(n: number): string {
  if (n >= 1000) return `$${n.toFixed(0)}`;
  if (n >= 100) return `$${n.toFixed(1)}`;
  return `$${n.toFixed(2)}`;
}

// `store.ts`'s own `parseStatsDate` builds every `Date` here from LOCAL y/m/d components (its own
// header comment explains why). Turning one of those back into a `yyyy-MM-dd` key has the
// identical pitfall in the other direction: `.toISOString()` reads the UTC wall-clock, so for any
// positive UTC offset (most of Europe, Africa, Asia, Australia) local midnight is still the
// *previous* UTC day, silently shifting every date key back by one. Local getters only.
function localDateKey(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

// --- Theme-aware palette (6 fixed slots, no wraparound -- analytics.md §5) ---
const PALETTE_VARS = ['--accent', '--color-blue', '--color-purple', '--color-teal', '--color-green', '--color-orange'] as const;
const NEUTRAL_FALLBACK = '#8E8E93';

function resolveVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

const palette = computed<string[]>(() => {
  void themeStore.current.id; // reactive dependency: re-read the CSS custom properties whenever the active theme changes
  return PALETTE_VARS.map(resolveVar);
});
const textSecondary = computed(() => {
  void themeStore.current.id;
  return resolveVar('--text-secondary');
});
const textTertiary = computed(() => {
  void themeStore.current.id;
  return resolveVar('--text-tertiary');
});
const borderColor = computed(() => {
  void themeStore.current.id;
  return resolveVar('--border');
});
const accentColor = computed(() => {
  void themeStore.current.id;
  return resolveVar('--accent');
});
const bgCardColor = computed(() => {
  void themeStore.current.id;
  return resolveVar('--bg-card');
});

const allModelNames = computed<string[]>(() => {
  const names = new Set<string>();
  for (const e of store.costBreakdownEntries) names.add(e.model);
  for (const e of store.tokenTimeSeriesData) names.add(e.model);
  return [...names].sort();
});
function colorForModel(model: string): string {
  const idx = allModelNames.value.indexOf(model);
  return idx >= 0 && idx < palette.value.length ? palette.value[idx] : NEUTRAL_FALLBACK;
}

// --- Daily Activity chart (§7.1): bar, range-filtered, accent fill ---
const dailyActivityOption = computed<EChartsOption>(() => ({
  grid: { left: 40, right: 16, top: 16, bottom: 32 },
  xAxis: { type: 'category', data: store.filteredDailyActivity.map((d) => d.date ?? ''), axisLabel: { color: textTertiary.value, fontSize: 10 }, axisLine: { lineStyle: { color: borderColor.value } } },
  yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10 }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
  tooltip: { trigger: 'axis', valueFormatter: (v) => String(v) },
  series: [{ type: 'bar', name: 'Messages', data: store.filteredDailyActivity.map((d) => d.messageCount ?? 0), itemStyle: { color: accentColor.value, borderRadius: [2, 2, 0, 0] }, barMaxWidth: 16 }],
}));

// --- Activity by Hour chart (§7.2): bar, all-time, ignores selectedDateRange entirely ---
const hourlyActivityOption = computed<EChartsOption>(() => {
  const counts = store.stats?.hourCounts ?? {};
  const hours = Array.from({ length: 24 }, (_, h) => h);
  return {
    grid: { left: 40, right: 16, top: 16, bottom: 32 },
    xAxis: { type: 'category', data: hours.map((h) => (h === 0 ? '12AM' : h === 12 ? '12PM' : h > 12 ? `${h - 12}PM` : `${h}AM`)), axisLabel: { color: textTertiary.value, fontSize: 10, interval: (idx: number) => [0, 6, 12, 18, 23].includes(idx) }, axisLine: { lineStyle: { color: borderColor.value } } },
    yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10 }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
    tooltip: { trigger: 'axis' },
    series: [{ type: 'bar', name: 'Sessions', data: hours.map((h) => counts[String(h)] ?? 0), itemStyle: { color: accentColor.value, borderRadius: [3, 3, 0, 0] }, barMaxWidth: 12 }],
  };
});

// --- Model Usage donut (§7.3): all-time, tokens desc, stable per-model colour ---
const modelUsageOption = computed<EChartsOption>(() => {
  const data = Object.entries(store.stats?.modelUsage ?? {})
    .map(([id, v]) => ({ name: friendlyModelName(id), value: (v.inputTokens ?? 0) + (v.outputTokens ?? 0) }))
    .sort((a, b) => b.value - a.value);
  return {
    tooltip: { trigger: 'item', valueFormatter: (v) => formatLargeNumber(Number(v)) },
    legend: { bottom: 0, textStyle: { color: textSecondary.value, fontSize: 11 } },
    series: [{ type: 'pie', radius: ['60%', '85%'], itemStyle: { borderColor: bgCardColor.value, borderWidth: 2 }, label: { show: false }, data: data.map((d) => ({ ...d, itemStyle: { color: colorForModel(d.name) } })) }],
  };
});

// --- Token Usage Over Time (§7.5): range-filtered, stacked area+line per model, stable colour ---
const tokenUsageOverTimeOption = computed<EChartsOption>(() => {
  const dates = [...new Set(store.tokenTimeSeriesData.map((e) => localDateKey(e.date)))].sort();
  const series = allModelNames.value
    .filter((model) => store.tokenTimeSeriesData.some((e) => e.model === model))
    .map<LineSeriesOption>((model) => ({
      type: 'line',
      name: model,
      data: dates.map((d) => store.tokenTimeSeriesData.find((e) => e.model === model && localDateKey(e.date) === d)?.tokens ?? 0),
      areaStyle: { opacity: 0.3 },
      lineStyle: { width: 1.5, color: colorForModel(model) },
      itemStyle: { color: colorForModel(model) },
      smooth: true,
      showSymbol: false,
    }));
  return {
    grid: { left: 48, right: 16, top: 16, bottom: 48 },
    xAxis: { type: 'category', data: dates, axisLabel: { color: textTertiary.value, fontSize: 10 }, axisLine: { lineStyle: { color: borderColor.value } } },
    yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10, formatter: (v: number) => formatLargeNumber(v) }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
    tooltip: { trigger: 'axis', valueFormatter: (v) => formatLargeNumber(Number(v)) },
    legend: { bottom: 0, textStyle: { color: textSecondary.value, fontSize: 11 } },
    series,
  };
});

// --- Tool Calls Over Time (§7.6): range-filtered, teal area+line ---
const toolCallsOverTimeOption = computed<EChartsOption>(() => {
  const teal = palette.value[3];
  return {
    grid: { left: 40, right: 16, top: 16, bottom: 32 },
    xAxis: { type: 'category', data: store.filteredDailyActivity.map((d) => d.date ?? ''), axisLabel: { color: textTertiary.value, fontSize: 10 }, axisLine: { lineStyle: { color: borderColor.value } } },
    yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10 }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
    tooltip: { trigger: 'axis' },
    series: [{ type: 'line', name: 'Tool calls', data: store.filteredDailyActivity.map((d) => d.toolCallCount ?? 0), areaStyle: { color: teal, opacity: 0.15 }, lineStyle: { width: 2, color: teal }, itemStyle: { color: teal }, smooth: true, showSymbol: false }],
  };
});

// --- Cost Breakdown donut + token table (§7.4) ---
const costBreakdownOption = computed<EChartsOption>(() => ({
  tooltip: { trigger: 'item', valueFormatter: (v) => formatCost(Number(v)) },
  series: [
    {
      type: 'pie',
      radius: ['60%', '85%'],
      itemStyle: { borderColor: bgCardColor.value, borderWidth: 2 },
      label: { show: true, position: 'center', formatter: () => `Total\n${formatCost(store.totalCost)}`, color: textSecondary.value, fontSize: 13 },
      data: store.costBreakdownEntries.map((e) => ({ name: e.model, value: e.cost, itemStyle: { color: colorForModel(e.model) } })),
    },
  ],
}));

// --- Activity heatmap (§6): a plain coloured grid, not an ECharts chart -- the data's job here is
// cells, not a measured plot, and Swift's own week-since-first-day grid doesn't match a real
// calendar-month layout ECharts' own `calendar` coordinate would impose. Kept in this same lazy
// component (not AnalyticsDashboard.vue) purely for cohesion with the rest of the visualisation
// area and to share `localDateKey` above, despite needing no ECharts import of its own. ---
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
const DAY_LABELS = ['Mon', '', 'Wed', '', 'Fri', '', 'Sun'];
</script>

<template>
  <ChartCard
    title="Daily Activity"
    :height="220"
  >
    <VChart
      :option="dailyActivityOption"
      autoresize
    />
  </ChartCard>

  <ChartCard
    title="Activity by Hour"
    subtitle="Session start times"
    :height="200"
  >
    <VChart
      :option="hourlyActivityOption"
      autoresize
    />
  </ChartCard>

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
      <div class="flex gap-0.5 overflow-x-auto">
        <div
          v-for="(week, wi) in heatmapGrid"
          :key="wi"
          class="flex flex-col gap-0.5"
        >
          <div
            v-for="(cell, di) in week"
            :key="di"
            :title="cell.dateString ? `${cell.dateString}: ${cell.messages} messages · ${cell.sessions} sessions` : undefined"
            class="h-3 w-3 rounded-xs"
            :class="heatmapCellClass(cell)"
          />
        </div>
      </div>
    </div>
  </ChartCard>

  <ChartCard
    title="Token Usage Over Time"
    :height="240"
  >
    <VChart
      :option="tokenUsageOverTimeOption"
      autoresize
    />
  </ChartCard>

  <ChartCard
    title="Tool Calls Over Time"
    :height="200"
  >
    <VChart
      :option="toolCallsOverTimeOption"
      autoresize
    />
  </ChartCard>

  <div class="grid grid-cols-2 gap-(--spacing-md)">
    <ChartCard
      title="Model Usage"
      :height="200"
    >
      <VChart
        :option="modelUsageOption"
        autoresize
      />
    </ChartCard>

    <ChartCard
      title="Cost & Token Breakdown"
      :height="280"
    >
      <div
        v-if="!store.hasCostData"
        class="flex items-center gap-(--spacing-xs) rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(--text-secondary) text-(length:--text-small)"
      >
        <AppIcon
          name="info.circle"
          size="sm"
        />
        Cost tracking is available for API users only
      </div>
      <VChart
        v-else
        :option="costBreakdownOption"
        style="height: 160px"
        autoresize
      />
      <table class="mt-(--spacing-sm) w-full text-(length:--text-tiny)">
        <thead>
          <tr class="text-(--text-tertiary)">
            <th class="text-left font-normal">
              Model
            </th>
            <th class="text-right font-normal">
              Input
            </th>
            <th class="text-right font-normal">
              Output
            </th>
            <th class="text-right font-normal">
              Cache
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in store.costBreakdownEntries"
            :key="entry.model"
            class="text-(--text-primary)"
          >
            <td class="truncate">
              {{ entry.model }}
            </td>
            <td class="text-right">
              {{ formatLargeNumber(entry.inputTokens) }}
            </td>
            <td class="text-right">
              {{ formatLargeNumber(entry.outputTokens) }}
            </td>
            <td class="text-right">
              {{ formatLargeNumber(entry.cacheTokens) }}
            </td>
          </tr>
        </tbody>
      </table>
    </ChartCard>
  </div>
</template>
