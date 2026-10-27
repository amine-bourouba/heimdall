<script setup lang="ts">
// Ported from DailyActivityChart.swift (M3-33; docs/spec/analytics.md §7.1): bar, range-filtered,
// accent fill. Swift's own chart supports tap-to-select with a floating annotation
// (`chartXSelection`/`chartOverlay`) -- ECharts has no equivalent without substantial custom
// overlay work, so this uses ECharts' own idiomatic hover tooltip instead (an established
// substitution, not an oversight: every chart in this port does the same).
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
import ChartCard from '../components/ChartCard.vue';

use([BarChart, GridComponent, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<BarSeriesOption | GridComponentOption | TooltipComponentOption>;

const store = useAnalyticsStore();
const { textTertiary, borderColor, accentColor } = useEchartsTheme();

const option = computed<EChartsOption>(() => ({
  grid: { left: 40, right: 16, top: 16, bottom: 32 },
  xAxis: { type: 'category', data: store.filteredDailyActivity.map((d) => d.date ?? ''), axisLabel: { color: textTertiary.value, fontSize: 10 }, axisLine: { lineStyle: { color: borderColor.value } } },
  yAxis: { type: 'value', axisLabel: { color: textTertiary.value, fontSize: 10 }, splitLine: { lineStyle: { color: borderColor.value, type: 'dashed' } } },
  tooltip: { trigger: 'axis', valueFormatter: (v) => String(v) },
  series: [{ type: 'bar', name: 'Messages', data: store.filteredDailyActivity.map((d) => d.messageCount ?? 0), itemStyle: { color: accentColor.value, borderRadius: [2, 2, 0, 0] }, barMaxWidth: 16 }],
}));
</script>

<template>
  <ChartCard
    title="Daily Activity"
    :height="220"
  >
    <VChart
      :option="option"
      autoresize
    />
  </ChartCard>
</template>
