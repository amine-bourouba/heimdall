<script setup lang="ts">
// Ported from CostBreakdownView.swift (M3-33; docs/spec/analytics.md §7.4): cost donut + token
// table. Same escaping rationale as ModelUsageChart.vue -- `entry.model` reaching the token table
// below goes through Vue's own template interpolation (`{{ }}`), which auto-escapes on its own;
// only the ECharts tooltip (real HTML, not a Vue template) needs the explicit `encodeHTML` call.
import { computed } from 'vue';
import { use } from 'echarts/core';
import { PieChart } from 'echarts/charts';
import { TooltipComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import VChart from 'vue-echarts';
import type { ComposeOption } from 'echarts/core';
import type { PieSeriesOption } from 'echarts/charts';
import type { TooltipComponentOption } from 'echarts/components';
import { useAnalyticsStore } from '../store';
import { useEchartsTheme, colorForModel } from './echartsTheme';
import { escapeForTooltip, formatCost, formatLargeNumber } from './formatters';
import AppIcon from '@/components/AppIcon.vue';
import ChartCard from '../components/ChartCard.vue';

use([PieChart, TooltipComponent, CanvasRenderer]);

type EChartsOption = ComposeOption<PieSeriesOption | TooltipComponentOption>;

const store = useAnalyticsStore();
const { palette, textSecondary, bgCardColor } = useEchartsTheme();

function tooltipFormatter(params: unknown): string {
  const p = (Array.isArray(params) ? params[0] : params) as { name?: string; value?: unknown };
  const name = escapeForTooltip(String(p.name ?? ''));
  return `${name}: ${formatCost(Number(p.value ?? 0))}`;
}

const option = computed<EChartsOption>(() => ({
  tooltip: { trigger: 'item', formatter: tooltipFormatter },
  series: [
    {
      type: 'pie',
      radius: ['60%', '85%'],
      itemStyle: { borderColor: bgCardColor.value, borderWidth: 2 },
      label: { show: true, position: 'center', formatter: () => `Total\n${formatCost(store.totalCost)}`, color: textSecondary.value, fontSize: 13 },
      data: store.costBreakdownEntries.map((e) => ({ name: e.model, value: e.cost, itemStyle: { color: colorForModel(e.model, store.allModelNames, palette.value) } })),
    },
  ],
}));
</script>

<template>
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
      :option="option"
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
</template>
