// Ported from AnalyticsViewModel.swift (M3-31; docs/spec/analytics.md sections 1-4 are the
// authoritative citation for every formula below, each already cross-referenced against the
// Swift source file:line). Owns only the DTO -> derived-data transformation this spec actually
// assigns the view-model (range filtering, token series, heatmap data, cost breakdown, totals) --
// `isCustomRangePresented`/`customStartDate`/`customEndDate` (AnalyticsViewModel.swift:62-65) are
// skipped: no Swift test exercises them, they are popover-local UI state with no IPC write of
// their own, and `selectedDateRange` below is a plain settable ref a dashboard component can
// write directly (`analyticsStore.selectedDateRange = {kind: 'custom', start, end}`), the same
// direct-mutation idiom `sessionStore.sessionSearchQuery` already uses elsewhere in this port --
// add the popover state back here only if a future task finds it needs to be shared beyond one
// component. `get_stats` has no `Area` entry in `onDataChanged` (confirmed directly against
// Sidebar.vue's own `COUNTED_AREAS`), so this is a one-shot `load()` like `prefsStore`, not a
// live subscription.
import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc, IpcError } from '@/ipc/client';
import type { DailyActivity, DailyModelTokens, StatsCache, StatsSource } from '@/ipc/bindings';

export type DateRangePreset = 'week' | 'month' | 'quarter' | 'all';
export type DateRangeSelection = { kind: DateRangePreset } | { kind: 'custom'; start: Date; end: Date };

export interface TokenTimeSeriesEntry {
  date: Date;
  model: string;
  tokens: number;
}

export interface HeatmapEntry {
  date: Date;
  dateString: string;
  messages: number;
  sessions: number;
}

export interface CostBreakdownEntry {
  model: string;
  cost: number;
  inputTokens: number;
  outputTokens: number;
  cacheTokens: number;
}

/** `AnalyticsFormatters.parseDate` (analytics.md §4): `yyyy-MM-dd` parsed as *local* calendar
 * components, not `new Date(isoString)` -- a bare ISO date string parses as UTC midnight in JS,
 * which would silently shift every day-boundary comparison below by the local UTC offset. */
export function parseStatsDate(s: string): Date {
  const [y, m, d] = s.split('-').map(Number);
  return new Date(y, m - 1, d);
}

/** `AnalyticsFormatters.formatDuration` (analytics.md §4): `StatsCache.longestSessionFormatted`
 * duplicates this exact formula inline in Swift (separate implementation, same output shape) --
 * one function covers both `timeSavedFormatted` and `longestSessionFormatted` here instead. */
function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
}

/** `StatsCache.friendlyModelName` (analytics.md §4) -- order matters, first match wins, since
 * e.g. "sonnet-4-5".includes("sonnet-4") is also true. */
export function friendlyModelName(id: string): string {
  if (id.includes('opus-4-6')) return 'Opus 4.6';
  if (id.includes('opus-4-5')) return 'Opus 4.5';
  if (id.includes('sonnet-4-5')) return 'Sonnet 4.5';
  if (id.includes('sonnet-4')) return 'Sonnet 4';
  if (id.includes('haiku')) return 'Haiku';
  return id;
}

export const useAnalyticsStore = defineStore('analytics', () => {
  const stats = ref<StatsCache | null>(null);
  const source = ref<StatsSource | null>(null);
  const isLoading = ref(true);
  const selectedDateRange = ref<DateRangeSelection>({ kind: 'all' });

  const isCustomRange = computed(() => selectedDateRange.value.kind === 'custom');

  // analytics.md §2.1: the loaded stats' own `lastComputedDate`, not wall-clock "now" -- only
  // falls back to `.now` when there are no stats loaded at all.
  const referenceDate = computed<Date>(() => (stats.value?.lastComputedDate ? parseStatsDate(stats.value.lastComputedDate) : new Date()));

  // analytics.md §2.2, exact rule: .all -> null (no filtering at all); .custom -> used verbatim,
  // no normalisation/clamping.
  const dateRangeBounds = computed<{ start: Date; end: Date } | null>(() => {
    const sel = selectedDateRange.value;
    if (sel.kind === 'custom') return { start: sel.start, end: sel.end };
    if (sel.kind === 'all') return null;
    const days = { week: 7, month: 30, quarter: 90 }[sel.kind];
    const start = new Date(referenceDate.value);
    start.setDate(start.getDate() - days);
    return { start, end: referenceDate.value };
  });

  // analytics.md §2.3: inclusive on both ends; .all returns everything unfiltered.
  const filteredDailyActivity = computed<DailyActivity[]>(() => {
    if (!stats.value) return [];
    const activity = stats.value.dailyActivity ?? [];
    const range = dateRangeBounds.value;
    if (!range) return activity;
    return activity.filter((day) => day.date && parseStatsDate(day.date) >= range.start && parseStatsDate(day.date) <= range.end);
  });

  const filteredDailyModelTokens = computed<DailyModelTokens[]>(() => {
    if (!stats.value) return [];
    const tokens = stats.value.dailyModelTokens ?? [];
    const range = dateRangeBounds.value;
    if (!range) return tokens;
    return tokens.filter((day) => day.date && parseStatsDate(day.date) >= range.start && parseStatsDate(day.date) <= range.end);
  });

  // analytics.md §3.1: one entry per (day, model) pair, range-filtered via filteredDailyModelTokens.
  const tokenTimeSeriesData = computed<TokenTimeSeriesEntry[]>(() =>
    filteredDailyModelTokens.value.flatMap((day) =>
      Object.entries(day.tokensByModel ?? {}).map(([model, tokens]) => ({
        date: parseStatsDate(day.date!),
        model: friendlyModelName(model),
        tokens,
      })),
    ),
  );

  // analytics.md §3.2: dateString is the original yyyy-MM-dd string, preserved verbatim.
  const heatmapData = computed<HeatmapEntry[]>(() =>
    filteredDailyActivity.value.map((day) => ({
      date: parseStatsDate(day.date!),
      dateString: day.date!,
      messages: day.messageCount ?? 0,
      sessions: day.sessionCount ?? 0,
    })),
  );

  // analytics.md §3.3: all-time (stats.modelUsage, not range-filtered), sorted by cost descending.
  const costBreakdownEntries = computed<CostBreakdownEntry[]>(() => {
    if (!stats.value) return [];
    return Object.entries(stats.value.modelUsage ?? {})
      .map(([key, value]) => ({
        model: friendlyModelName(key),
        cost: value.costUSD ?? 0,
        inputTokens: value.inputTokens ?? 0,
        outputTokens: value.outputTokens ?? 0,
        cacheTokens: (value.cacheReadInputTokens ?? 0) + (value.cacheCreationInputTokens ?? 0),
      }))
      .sort((a, b) => b.cost - a.cost);
  });

  // analytics.md §12.3: computed once, here, so every chart (M3-33) shares the identical sorted
  // model-name set -- `colorForModel` looks a model up against this same array regardless of
  // which chart is asking, which is what keeps one model's colour stable across all of them
  // (the open question the Swift reference app's own per-chart re-slicing never resolved).
  const allModelNames = computed<string[]>(() => {
    const names = new Set<string>();
    for (const e of costBreakdownEntries.value) names.add(e.model);
    for (const e of tokenTimeSeriesData.value) names.add(e.model);
    return [...names].sort();
  });

  // analytics.md §3.4: Heimdall's own `StatsCache` DTO has no pre-computed `totalCostUSD`/
  // `totalInputTokens`/`totalOutputTokens` fields (confirmed directly in bindings.ts) -- those are
  // Swift-side computed properties on `StatsCache`, not raw JSON, so they're derived from
  // `modelUsage` here exactly as `StatsCache.swift`'s own extension does: input is
  // cache-inclusive (input + cacheRead + cacheCreation), output is not.
  const totalCost = computed(() => Object.values(stats.value?.modelUsage ?? {}).reduce((sum, v) => sum + (v.costUSD ?? 0), 0));
  const hasCostData = computed(() => totalCost.value > 0);
  const totalInputTokens = computed(() =>
    Object.values(stats.value?.modelUsage ?? {}).reduce((sum, v) => sum + (v.inputTokens ?? 0) + (v.cacheReadInputTokens ?? 0) + (v.cacheCreationInputTokens ?? 0), 0),
  );
  const totalOutputTokens = computed(() => Object.values(stats.value?.modelUsage ?? {}).reduce((sum, v) => sum + (v.outputTokens ?? 0), 0));
  const totalTokens = computed(() => totalInputTokens.value + totalOutputTokens.value);
  // All-time, across every day in the cache, not just the filtered range (analytics.md §3.4).
  const totalToolCalls = computed(() => (stats.value?.dailyActivity ?? []).reduce((sum, day) => sum + (day.toolCallCount ?? 0), 0));

  const hasTimeSavedData = computed(() => (stats.value?.totalSpeculationTimeSavedMs ?? 0) > 0);
  const timeSavedFormatted = computed(() => {
    const ms = stats.value?.totalSpeculationTimeSavedMs;
    if (!stats.value || !ms) return '—';
    return formatDuration(ms);
  });

  // docs/tasks/M3-31's own Input: StatsCacheTests.swift's `longestSessionFormatted` case -- a
  // StatsCache-level computed in Swift, exposed from the store here since this is the one
  // Vue-facing place the dashboard (M3-32) reads display-ready stats from.
  const longestSessionFormatted = computed(() => {
    const duration = stats.value?.longestSession?.duration;
    return duration ? formatDuration(duration) : '—';
  });

  async function load() {
    isLoading.value = true;
    try {
      const view = await ipc.getStats();
      stats.value = view.stats;
      source.value = view.source;
    } catch (err) {
      // "no stats cache and nothing to compute from (no sessions yet)" (analytics.rs) -- the
      // real, expected empty state (Swift's own nil-stats case), not swallowed silently: any
      // other error kind still propagates (ADR 0005, analytics.md §1.1's own "must not swallow").
      if (err instanceof IpcError && err.kind === 'notFound') {
        stats.value = null;
        source.value = null;
      } else {
        throw err;
      }
    } finally {
      isLoading.value = false;
    }
  }

  return {
    stats,
    source,
    isLoading,
    selectedDateRange,
    isCustomRange,
    referenceDate,
    filteredDailyActivity,
    filteredDailyModelTokens,
    tokenTimeSeriesData,
    heatmapData,
    costBreakdownEntries,
    allModelNames,
    totalCost,
    hasCostData,
    totalTokens,
    totalToolCalls,
    timeSavedFormatted,
    hasTimeSavedData,
    longestSessionFormatted,
    load,
  };
});
