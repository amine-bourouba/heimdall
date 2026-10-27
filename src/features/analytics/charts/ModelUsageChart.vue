<script setup lang="ts">
// Ported from ModelUsageChart.swift (M3-33; docs/spec/analytics.md §7.3): donut, all-time, tokens
// desc, stable per-model colour (analytics.md §12.3 -- see `echartsTheme.ts`'s own header comment
// for why this deliberately deviates from Swift's own per-chart palette re-slicing).
//
// Security (ADR 0010 control 1, M3-33 AC2): `friendlyModelName()` falls back to the raw model-id
// string for anything it doesn't recognise, and that id traces back to a JSONL `model` field on
// disk -- attacker-influenceable, same threat model as every other disk-read string this app
// renders. ECharts' tooltip is real DOM/HTML, not a Vue template, so Vue's own auto-escaping does
// not apply to it: the pie's `name` field is kept raw (ECharts draws legends on canvas as plain
// text, so a pre-escaped name would show literal "&lt;" to the user there), and the escaping
// happens only inside the tooltip's own `formatter`, which builds the HTML string by hand.
import { computed } from 'vue';
import { use } from 'echarts/core';
import { PieChart } from 'echarts/charts';
import { LegendComponent, TooltipComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import type { TooltipComponentOption } from 'echarts/components';
import VChart from 'vue-echarts';
import type { ComposeOption } from 'echarts/core';
import type { PieSeriesOption } from 'echarts/charts';
import type { LegendComponentOption } from 'echarts/components';
import { useAnalyticsStore, friendlyModelName } from '../store';
import { useEchartsTheme, colorForModel } from './echartsTheme';
import { escapeForTooltip, formatLargeNumber } from './formatters';
import ChartCard from '../components/ChartCard.vue';

use([PieChart, LegendComponent, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<PieSeriesOption | LegendComponentOption | TooltipComponentOption>;

const store = useAnalyticsStore();
const { palette, textSecondary, bgCardColor } = useEchartsTheme();

function tooltipFormatter(params: unknown): string {
  const p = (Array.isArray(params) ? params[0] : params) as { name?: string; value?: unknown; percent?: number };
  const name = escapeForTooltip(String(p.name ?? ''));
  const value = formatLargeNumber(Number(p.value ?? 0));
  return p.percent === undefined ? `${name}: ${value}` : `${name}: ${value} (${p.percent}%)`;
}

const option = computed<EChartsOption>(() => {
  const data = Object.entries(store.stats?.modelUsage ?? {})
    .map(([id, v]) => ({ name: friendlyModelName(id), value: (v.inputTokens ?? 0) + (v.outputTokens ?? 0) }))
    .sort((a, b) => b.value - a.value);
  return {
    tooltip: { trigger: 'item', formatter: tooltipFormatter },
    legend: { bottom: 0, textStyle: { color: textSecondary.value, fontSize: 11 } },
    series: [{ type: 'pie', radius: ['60%', '85%'], itemStyle: { borderColor: bgCardColor.value, borderWidth: 2 }, label: { show: false }, data: data.map((d) => ({ ...d, itemStyle: { color: colorForModel(d.name, store.allModelNames, palette.value) } })) }],
  };
});
</script>

<template>
  <ChartCard
    title="Model Usage"
    :height="200"
  >
    <VChart
      :option="option"
      autoresize
    />
  </ChartCard>
</template>
