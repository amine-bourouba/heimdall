// Ported from PoirotTests/Views/AnalyticsViewModelTests.swift (15 cases) plus
// PoirotTests/Models/StatsCacheTests.swift's own `longestSessionFormatted_showsHoursAndMinutes`
// case (docs/spec/test-carryover.md), 16 total. AC: "pass with a fixed clock" -- `referenceDate`
// falls back to the real `new Date()` only when no stats are loaded at all (every other case here
// anchors off the fixture's own `lastComputedDate`), but the clock is faked for every test anyway,
// matching this task's own AC wording literally and `history/store.spec.ts`'s established precedent.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import type { DailyActivity, DailyModelTokens, ModelUsage, StatsCache } from '@/ipc/bindings';
import { useAnalyticsStore } from './store';

const NOW = new Date('2026-02-20T12:00:00');

function makeStats(overrides: Partial<StatsCache> = {}): StatsCache {
  return {
    version: 2,
    lastComputedDate: '2026-02-17',
    dailyActivity: [],
    dailyModelTokens: [],
    modelUsage: {},
    totalSessions: 10,
    totalMessages: 100,
    longestSession: { sessionId: 'test', duration: 3_600_000, messageCount: 50, timestamp: '2026-01-01T00:00:00.000Z' },
    firstSessionDate: '2025-12-31T19:57:52.149Z',
    hourCounts: { '9': 5, '14': 10 },
    totalSpeculationTimeSavedMs: 0,
    ...overrides,
  };
}

const sampleActivity: DailyActivity[] = [
  { date: '2026-01-01', messageCount: 100, sessionCount: 5, toolCallCount: 50 },
  { date: '2026-01-15', messageCount: 200, sessionCount: 8, toolCallCount: 120 },
  { date: '2026-02-01', messageCount: 300, sessionCount: 10, toolCallCount: 80 },
  { date: '2026-02-15', messageCount: 150, sessionCount: 3, toolCallCount: 40 },
];

const sampleModelTokens: DailyModelTokens[] = [
  { date: '2026-01-01', tokensByModel: { 'claude-opus-4-5-20251101': 1000 } },
  { date: '2026-02-01', tokensByModel: { 'claude-opus-4-5-20251101': 2000, 'claude-opus-4-6': 500 } },
];

const sampleModelUsage: Record<string, ModelUsage> = {
  'claude-opus-4-5-20251101': { inputTokens: 1000, outputTokens: 2000, cacheReadInputTokens: 500, cacheCreationInputTokens: 300, webSearchRequests: 0, costUSD: 15.5, contextWindow: 200_000 },
  'claude-opus-4-6': { inputTokens: 800, outputTokens: 1500, cacheReadInputTokens: 200, cacheCreationInputTokens: 100, webSearchRequests: 0, costUSD: 10.25, contextWindow: 200_000 },
};

async function loadedStore(stats: StatsCache | null) {
  mockIPC((cmd) => {
    if (cmd === 'get_stats') {
      if (stats === null) throw { kind: 'notFound', message: 'no stats', path: null };
      return { stats, source: 'cache' };
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const store = useAnalyticsStore();
  await store.load();
  return store;
}

beforeEach(() => {
  setActivePinia(createPinia());
  vi.useFakeTimers();
  vi.setSystemTime(NOW);
});

afterEach(() => {
  vi.useRealTimers();
});

describe('analytics store: date range filtering (filteredDailyActivity_*)', () => {
  it('allRange_returnsAll', async () => {
    const store = await loadedStore(makeStats({ dailyActivity: sampleActivity }));
    store.selectedDateRange = { kind: 'all' };

    expect(store.filteredDailyActivity).toHaveLength(4);
  });

  it('weekRange_filtersOldEntries', async () => {
    // Dates relative to lastComputedDate ("2026-02-17") used in makeStats
    const recentStr = '2026-02-15'; // 2 days before lastComputed -> within 7d
    const oldStr = '2026-01-18'; // 30 days before lastComputed -> outside 7d
    const store = await loadedStore(
      makeStats({
        dailyActivity: [
          { date: recentStr, messageCount: 10, sessionCount: 1, toolCallCount: 5 },
          { date: oldStr, messageCount: 20, sessionCount: 2, toolCallCount: 10 },
        ],
      }),
    );
    store.selectedDateRange = { kind: 'week' };

    expect(store.filteredDailyActivity).toHaveLength(1);
    expect(store.filteredDailyActivity[0].date).toBe(recentStr);
  });
});

describe('analytics store: token time series (tokenTimeSeriesData_*)', () => {
  it('flatMapsModels', async () => {
    const store = await loadedStore(makeStats({ dailyModelTokens: sampleModelTokens }));
    store.selectedDateRange = { kind: 'all' };

    // First day: 1 model, second day: 2 models = 3 entries
    expect(store.tokenTimeSeriesData).toHaveLength(3);
  });

  it('usesFriendlyNames', async () => {
    const store = await loadedStore(makeStats({ dailyModelTokens: sampleModelTokens }));
    store.selectedDateRange = { kind: 'all' };

    const models = new Set(store.tokenTimeSeriesData.map((e) => e.model));
    expect(models.has('Opus 4.5')).toBe(true);
    expect(models.has('Opus 4.6')).toBe(true);
  });
});

describe('analytics store: cost breakdown (costBreakdownEntries_*)', () => {
  it('sortedByCostDescending', async () => {
    const store = await loadedStore(makeStats({ modelUsage: sampleModelUsage }));

    const entries = store.costBreakdownEntries;
    expect(entries).toHaveLength(2);
    expect(entries[0].cost).toBeGreaterThanOrEqual(entries[1].cost);
  });
});

describe('analytics store: summary computations', () => {
  it('totalCost_sumsModelCosts', async () => {
    const store = await loadedStore(makeStats({ modelUsage: sampleModelUsage }));

    expect(store.totalCost).toBe(15.5 + 10.25);
  });

  it('totalTokens_sumsInputAndOutput', async () => {
    const store = await loadedStore(makeStats({ modelUsage: sampleModelUsage }));

    // Input (cache-inclusive): (1000 + 500 + 300) + (800 + 200 + 100) = 2900
    // Output: 2000 + 1500 = 3500
    expect(store.totalTokens).toBe(2900 + 3500);
  });

  it('totalToolCalls_sumsDailyActivity', async () => {
    const store = await loadedStore(makeStats({ dailyActivity: sampleActivity }));

    expect(store.totalToolCalls).toBe(50 + 120 + 80 + 40);
  });

  it('timeSavedFormatted_displaysCorrectly', async () => {
    const store = await loadedStore(makeStats({ totalSpeculationTimeSavedMs: 3_660_000 })); // 1h 1m

    expect(store.timeSavedFormatted).toBe('1h 1m');
  });

  it('timeSavedFormatted_zeroMs_showsDash', async () => {
    const store = await loadedStore(makeStats({ totalSpeculationTimeSavedMs: 0 }));

    expect(store.timeSavedFormatted).toBe('—');
    expect(store.hasTimeSavedData).toBe(false);
  });

  it('hasCostData_zeroCost_returnsFalse', async () => {
    const store = await loadedStore(makeStats({ modelUsage: {} }));

    expect(store.hasCostData).toBe(false);
  });

  it('hasCostData_withCost_returnsTrue', async () => {
    const store = await loadedStore(makeStats({ modelUsage: sampleModelUsage }));

    expect(store.hasCostData).toBe(true);
  });
});

describe('analytics store: custom date range (customRange_filtersCorrectly)', () => {
  it('filters inclusively between start and end, and flags isCustomRange', async () => {
    const store = await loadedStore(
      makeStats({
        dailyActivity: [
          { date: '2026-01-01', messageCount: 10, sessionCount: 1, toolCallCount: 5 },
          { date: '2026-01-15', messageCount: 20, sessionCount: 2, toolCallCount: 10 },
          { date: '2026-02-01', messageCount: 30, sessionCount: 3, toolCallCount: 15 },
        ],
      }),
    );

    store.selectedDateRange = { kind: 'custom', start: new Date(2026, 0, 10), end: new Date(2026, 0, 20) };

    expect(store.filteredDailyActivity).toHaveLength(1);
    expect(store.filteredDailyActivity[0].date).toBe('2026-01-15');
    expect(store.isCustomRange).toBe(true);
  });
});

describe('analytics store: heatmap data (heatmapData_mapsActivityEntries)', () => {
  it('maps activity entries to heatmap entries', async () => {
    const store = await loadedStore(makeStats({ dailyActivity: sampleActivity }));
    store.selectedDateRange = { kind: 'all' };

    const heatmap = store.heatmapData;
    expect(heatmap).toHaveLength(4);
    expect(heatmap[0].messages).toBe(100);
    expect(heatmap[0].sessions).toBe(5);
    expect(heatmap[0].dateString).toBe('2026-01-01');
  });
});

describe('analytics store: no stats loaded (filteredDailyActivity_nilStats_returnsEmpty)', () => {
  it('every derived value is empty/zero when get_stats finds nothing (not found)', async () => {
    const store = await loadedStore(null);

    expect(store.filteredDailyActivity).toEqual([]);
    expect(store.tokenTimeSeriesData).toEqual([]);
    expect(store.heatmapData).toEqual([]);
    expect(store.costBreakdownEntries).toEqual([]);
    expect(store.totalCost).toBe(0);
    expect(store.totalTokens).toBe(0);
    expect(store.totalToolCalls).toBe(0);
  });
});

describe('analytics store: longestSessionFormatted (StatsCacheTests.swift)', () => {
  it('showsHoursAndMinutes', async () => {
    // 321972719 ms = ~89h 26m
    const store = await loadedStore(makeStats({ longestSession: { sessionId: 'x', duration: 321_972_719, messageCount: 1, timestamp: '2026-01-01T00:00:00.000Z' } }));

    expect(store.longestSessionFormatted).toBe('89h 26m');
  });
});
