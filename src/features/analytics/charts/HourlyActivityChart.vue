<script setup lang="ts">
// Ported from HourlyActivityChart.swift (M3-33; docs/spec/analytics.md §7.2): bar, all-time,
// ignores `selectedDateRange` entirely. Subtitle deliberately drops Swift's own "(UTC)" suffix --
// an already-flagged Swift reference inconsistency (the buckets are local-time, not UTC; see
// progress-log.md's M3-31 entry), not carried over.
import { computed } from 'vue';
import { use } from 'echarts/core';
import { BarChart } from 'echarts/charts';
import { GridComponent, TooltipComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import type { ComposeOption } from 'echarts/core';
import type { BarSeriesOption } from 'echarts/charts';
import type { GridComponentOption, TooltipComponentOption } from 'echarts/components';
import { useAnalyticsStore } from '../store';
import { useEchartsTheme } from './echartsTheme';
import { formatHour } from './formatters';
import ChartCard from '../components/ChartCard.vue';

use([BarChart, GridComponent, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<BarSeriesOption | GridComponentOption | TooltipComponentOption>;

const store = useAnalyticsStore();
const { textTertiary, borderColor, accentColor } = useEchartsTheme();

const option = computed<EChartsOption>(() => {
  const counts = store.stats?.hourCounts ?? {};
  const hours = Array.from({ length: 24 }, (_, h) => h);
  return {
    grid: { left: 40, right: 16, top: 16, bottom: 32 },
    xAxis: { type: 'category', data: hours.map(formatHour), axisLabel: { color: textTertiary.value, fontSize: 10, interval: (idx: number) => [0, 6, 12, 18, 23].includes(idx) }, axisLine: { lineStyle: { color: borderColor.value } } },
    yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10 }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
    tooltip: { trigger: 'axis' },
    series: [{ type: 'bar', name: 'Sessions', data: hours.map((h) => counts[String(h)] ?? 0), itemStyle: { color: accentColor.value, borderRadius: [3, 3, 0, 0] }, barMaxWidth: 12 }],
  };
});
</script>

<template>
  <ChartCard
    title="Activity by Hour"
    subtitle="Session start times"
    :height="200"
  >
    <VChart
      :option="option"
      autoresize
    />
  </ChartCard>
</template>
