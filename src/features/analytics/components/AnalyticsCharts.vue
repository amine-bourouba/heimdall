<script setup lang="ts">
// Thin composition root for the 6 charts (M3-33) + heatmap (M3-34), lazy-loaded from
// AnalyticsDashboard.vue via `defineAsyncComponent` (M3-32's bundle-size fix -- this file's own
// import path/name stays put so that wiring doesn't need to change). Each chart now owns its own
// ECharts registration, options and (where it touches a model name) `encodeHTML` escaping --
// nothing chart-specific lives here any more, see `../charts/` for all of that.
import ContributionHeatmap from './ContributionHeatmap.vue';
import DailyActivityChart from '../charts/DailyActivityChart.vue';
import HourlyActivityChart from '../charts/HourlyActivityChart.vue';
import TokenUsageOverTimeChart from '../charts/TokenUsageOverTimeChart.vue';
import ToolCallsOverTimeChart from '../charts/ToolCallsOverTimeChart.vue';
import ModelUsageChart from '../charts/ModelUsageChart.vue';
import CostBreakdownChart from '../charts/CostBreakdownChart.vue';
</script>

<template>
  <DailyActivityChart />
  <HourlyActivityChart />
  <ContributionHeatmap />
  <TokenUsageOverTimeChart />
  <ToolCallsOverTimeChart />

  <div class="grid grid-cols-2 gap-(--spacing-md)">
    <ModelUsageChart />
    <CostBreakdownChart />
  </div>
</template>
