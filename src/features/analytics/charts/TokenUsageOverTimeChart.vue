<script setup lang="ts">
// Ported from TokenUsageOverTimeChart.swift (M3-33; docs/spec/analytics.md §7.5): range-filtered,
// stacked area+line per model, stable per-model colour (see ModelUsageChart.vue's own header
// comment for the colour-stability rationale and CostBreakdownChart.vue's for the escaping one --
// the same applies here: series `name` is a model name, escaped only inside the tooltip
// `formatter`, never in the raw `name` field legends draw on canvas).
import { computed } from 'vue';
import { use } from 'echarts/core';
import { LineChart } from 'echarts/charts';
import { GridComponent, LegendComponent, TooltipComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import type { ComposeOption } from 'echarts/core';
import type { LineSeriesOption } from 'echarts/charts';
import type { GridComponentOption, LegendComponentOption, TooltipComponentOption } from 'echarts/components';
import { useAnalyticsStore } from '../store';
import { useEchartsTheme, colorForModel } from './echartsTheme';
import { escapeForTooltip, formatLargeNumber, localDateKey } from './formatters';
import ChartCard from '../components/ChartCard.vue';

use([LineChart, GridComponent, LegendComponent, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<LineSeriesOption | GridComponentOption | LegendComponentOption | TooltipComponentOption>;

const store = useAnalyticsStore();
const { palette, textSecondary, textTertiary, borderColor } = useEchartsTheme();

interface AxisTooltipRow {
  value?: unknown;
  axisValueLabel?: string;
  marker?: string;
  seriesName?: string;
}

function tooltipFormatter(params: unknown): string {
  const rows = (Array.isArray(params) ? params : [params]) as AxisTooltipRow[];
  const withValues = rows.filter((p) => p.value !== undefined && p.value !== null);
  if (withValues.length === 0) return '';
  const header = withValues[0].axisValueLabel ?? '';
  const lines = withValues.map((p) => `${p.marker ?? ''}${escapeForTooltip(String(p.seriesName ?? ''))}: ${formatLargeNumber(Number(p.value))}`);
  return [header, ...lines].join('<br/>');
}

const option = computed<EChartsOption>(() => {
  const dates = [...new Set(store.tokenTimeSeriesData.map((e) => localDateKey(e.date)))].sort();
  const series = store.allModelNames
    .filter((model) => store.tokenTimeSeriesData.some((e) => e.model === model))
    .map<LineSeriesOption>((model) => ({
      type: 'line',
      name: model,
      data: dates.map((d) => store.tokenTimeSeriesData.find((e) => e.model === model && localDateKey(e.date) === d)?.tokens ?? 0),
      areaStyle: { opacity: 0.3 },
      lineStyle: { width: 1.5, color: colorForModel(model, store.allModelNames, palette.value) },
      itemStyle: { color: colorForModel(model, store.allModelNames, palette.value) },
      smooth: true,
      showSymbol: false,
    }));
  return {
    grid: { left: 48, right: 16, top: 16, bottom: 48 },
    xAxis: { type: 'category', data: dates, axisLabel: { color: textTertiary.value, fontSize: 10 }, axisLine: { lineStyle: { color: borderColor.value } } },
    yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10, formatter: (v: number) => formatLargeNumber(v) }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
    tooltip: { trigger: 'axis', formatter: tooltipFormatter },
    legend: { bottom: 0, textStyle: { color: textSecondary.value, fontSize: 11 } },
    series,
  };
});
</script>

<template>
  <ChartCard
    title="Token Usage Over Time"
    :height="240"
  >
    <VChart
      :option="option"
      autoresize
    />
  </ChartCard>
</template>
