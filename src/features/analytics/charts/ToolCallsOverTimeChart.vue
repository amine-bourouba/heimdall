<script setup lang="ts">
// Ported from ToolCallsOverTimeChart.swift (M3-33; docs/spec/analytics.md §7.6): range-filtered,
// teal area+line. Single hardcoded series name ("Tool calls") -- no data-derived string reaches
// the tooltip, so no encodeHTML is needed here (contrast ModelUsageChart.vue/
// TokenUsageOverTimeChart.vue/CostBreakdownChart.vue, which all echo model names).
import { computed } from 'vue';
import { use } from 'echarts/core';
import { LineChart } from 'echarts/charts';
import { GridComponent, TooltipComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import type { ComposeOption } from 'echarts/core';
import type { LineSeriesOption } from 'echarts/charts';
import type { GridComponentOption, TooltipComponentOption } from 'echarts/components';
import { useAnalyticsStore } from '../store';
import { useEchartsTheme } from './echartsTheme';
import ChartCard from '../components/ChartCard.vue';

use([LineChart, GridComponent, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<LineSeriesOption | GridComponentOption | TooltipComponentOption>;

const store = useAnalyticsStore();
const { palette, textTertiary, borderColor } = useEchartsTheme();

const option = computed<EChartsOption>(() => {
  const teal = palette.value[3];
  return {
    grid: { left: 40, right: 16, top: 16, bottom: 32 },
    xAxis: { type: 'category', data: store.filteredDailyActivity.map((d) => d.date ?? ''), axisLabel: { color: textTertiary.value, fontSize: 10 }, axisLine: { lineStyle: { color: borderColor.value } } },
    yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10 }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
    tooltip: { trigger: 'axis' },
    series: [{ type: 'line', name: 'Tool calls', data: store.filteredDailyActivity.map((d) => d.toolCallCount ?? 0), areaStyle: { color: teal, opacity: 0.15 }, lineStyle: { width: 2, color: teal }, itemStyle: { color: teal }, smooth: true, showSymbol: false }],
  };
});
</script>

<template>
  <ChartCard
    title="Tool Calls Over Time"
    :height="200"
  >
    <VChart
      :option="option"
      autoresize
    />
  </ChartCard>
</template>
