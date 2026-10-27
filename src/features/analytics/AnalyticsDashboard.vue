<script setup lang="ts">
// Ported from AnalyticsDashboardView.swift (M3-32; docs/spec/analytics.md is the authoritative
// citation for every formula/layout decision below). Orchestrates the store (M3-31), the 8 stat
// cards and the CSV export menu directly; the 6 charts and the activity heatmap live in
// `AnalyticsCharts.vue`, loaded via `defineAsyncComponent` rather than a static import.
//
// That split is itself a necessary ripple, not the original design: ECharts + vue-echarts alone
// pushed this route to 197.64 KB gzip against the 150 KB lazy-route budget (port-plan.md §9),
// confirmed directly against a real build (`node scripts/check-bundle-size.mjs`), the same
// bundle-size regression class M3-12 already fixed once for Shiki with the identical
// `defineAsyncComponent` technique. The stat cards/CSV menu/loading/no-data states need no
// charting library at all, so they stay eager; only the real chart weight is deferred.
import { computed, defineAsyncComponent, onMounted, ref } from 'vue';
import { ipc, IpcError } from '@/ipc/client';
import { useToastsStore } from '@/stores/toasts';
import type { CsvKind } from '@/ipc/bindings';
import { useAnalyticsStore } from './store';
import AppIcon from '@/components/AppIcon.vue';
import StatCard from './components/StatCard.vue';
import AnalyticsShimmer from './components/AnalyticsShimmer.vue';
import DateRangePicker from './components/DateRangePicker.vue';

const AnalyticsCharts = defineAsyncComponent(() => import('./components/AnalyticsCharts.vue'));

const store = useAnalyticsStore();
const toastsStore = useToastsStore();

onMounted(() => void store.load());

// --- Formatters (AnalyticsFormatters.swift, analytics.md §4) -- also duplicated (small, pure) in
// AnalyticsCharts.vue, which needs them too but must not statically import anything from this
// eager chunk's own module graph. ---
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
function formatFirstSessionDate(iso: string | undefined): string {
  if (!iso) return 'Unknown';
  return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

// --- Stat cards (8, analytics.md §8) ---
const statCards = computed(() => {
  const stats = store.stats;
  if (!stats) return [];
  return [
    { title: 'Total Sessions', value: String(stats.totalSessions ?? 0), icon: 'rectangle.stack.fill', color: 'accent' as const },
    { title: 'Total Messages', value: formatLargeNumber(stats.totalMessages ?? 0), icon: 'message.fill', color: 'blue' as const },
    { title: 'Longest Session', value: store.longestSessionFormatted, subtitle: `${stats.longestSession?.messageCount ?? 0} messages`, icon: 'timer', color: 'orange' as const },
    { title: 'First Session', value: formatFirstSessionDate(stats.firstSessionDate), icon: 'calendar', color: 'green' as const },
    {
      title: 'Total Cost',
      value: store.hasCostData ? formatCost(store.totalCost) : '—',
      subtitle: store.hasCostData ? null : 'included in subscription',
      icon: 'dollarsign.circle.fill',
      color: 'green' as const,
      dimmed: !store.hasCostData,
      infoTooltip: store.hasCostData ? null : 'API users see per-model costs here. Subscription plans (Max, Pro) include usage at no extra charge.',
    },
    { title: 'Total Tokens', value: formatLargeNumber(store.totalTokens), icon: 'number.circle.fill', color: 'purple' as const },
    { title: 'Tool Calls', value: formatLargeNumber(store.totalToolCalls), icon: 'wrench.and.screwdriver.fill', color: 'teal' as const },
    {
      title: 'Time Saved',
      value: store.timeSavedFormatted,
      subtitle: store.hasTimeSavedData ? 'speculation cache' : 'no cache data',
      icon: 'clock.arrow.2.circlepath',
      color: 'blue' as const,
      dimmed: !store.hasTimeSavedData,
    },
  ];
});

// --- CSV export menu (analytics.md §9: all-time data, server-side; order is declaration order) ---
const CSV_KINDS: { kind: CsvKind; label: string }[] = [
  { kind: 'dailyActivity', label: 'Daily Activity' },
  { kind: 'modelUsage', label: 'Model Usage' },
  { kind: 'dailyTokens', label: 'Daily Tokens' },
  { kind: 'hourlyActivity', label: 'Hourly Activity' },
];
const isCsvMenuOpen = ref(false);

async function exportCsv(kind: CsvKind) {
  isCsvMenuOpen.value = false;
  try {
    const outcome = await ipc.exportAnalyticsCsv(kind);
    if (outcome === 'written') toastsStore.show('CSV exported');
  } catch (err) {
    // Swift's own exporter silently swallows write failures (`try?`) -- ADR 0005 forbids that
    // here, so a real error surfaces as a toast instead of vanishing (analytics.md §9.5).
    toastsStore.show(err instanceof IpcError ? err.message : 'Export failed', { style: 'error' });
  }
}
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto px-(--spacing-xxxl) py-(--spacing-xl)">
    <AnalyticsShimmer v-if="store.isLoading" />

    <template v-else-if="!store.stats">
      <div class="flex flex-1 flex-col items-center justify-center gap-(--spacing-sm) text-center text-(--text-tertiary)">
        <AppIcon
          name="chart.xyaxis.line"
          size="lg"
        />
        <h2 class="text-(--text-primary) text-(length:--text-heading-sm)">
          No session analytics yet
        </h2>
        <p class="max-w-100 text-(length:--text-small)">
          No Claude Code sessions found yet. Analytics are computed locally from your sessions and appear once you've used Claude Code in a project.
        </p>
      </div>
    </template>

    <div
      v-else
      class="flex flex-col gap-(--spacing-lg)"
    >
      <div class="flex items-center justify-between">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          Analytics
        </h1>
        <div class="flex items-center gap-(--spacing-sm)">
          <DateRangePicker />
          <div class="relative">
            <button
              type="button"
              title="Export CSV"
              class="flex items-center gap-(--spacing-xxs) rounded-sm border px-(--spacing-sm) py-(--spacing-xxs) text-(--text-secondary) text-(length:--text-small)"
              @click="isCsvMenuOpen = !isCsvMenuOpen"
            >
              <AppIcon
                name="square.and.arrow.down"
                size="sm"
              />
              Export CSV
            </button>
            <ul
              v-if="isCsvMenuOpen"
              role="menu"
              aria-label="Export CSV"
              class="glass absolute top-full right-0 z-10 mt-(--spacing-xs) min-w-40 rounded-md p-(--spacing-xs)"
            >
              <li
                v-for="item in CSV_KINDS"
                :key="item.kind"
                role="menuitem"
              >
                <button
                  type="button"
                  class="w-full rounded-sm px-(--spacing-sm) py-(--spacing-xs) text-left text-(--text-primary) text-(length:--text-small)"
                  @click="exportCsv(item.kind)"
                >
                  {{ item.label }}
                </button>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-4 gap-(--spacing-md)">
        <StatCard
          v-for="card in statCards"
          :key="card.title"
          v-bind="card"
        />
      </div>

      <AnalyticsCharts />
    </div>
  </div>
</template>
