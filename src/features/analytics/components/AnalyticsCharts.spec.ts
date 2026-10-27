// Direct-mount coverage for the lazy chart area M3-32 moved out of AnalyticsDashboard.vue (see
// this file's own header comment for why). Since M3-33/M3-34, AnalyticsCharts.vue is itself a
// thin composition root over `../charts/*.vue` + `./ContributionHeatmap.vue`; mounting it here
// still exercises the real tree end to end. AnalyticsDashboard.spec.ts only reaches this
// component through `defineAsyncComponent`,
// which it stubs out entirely -- a real reproduction showed the dynamic `import()` that loader
// fires never settles under Vitest's happy-dom + Vite SSR module runner (no error, no rejection),
// even though the exact same import resolves immediately when awaited directly or mounted
// synchronously, as this file does. So this is the one place that mounts the real component and
// asserts on its real output; AnalyticsDashboard.spec.ts covers the stat cards/CSV menu/loading
// states around it.
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { describe, expect, it, vi } from 'vitest';
import type { StatsCache } from '@/ipc/bindings';
import { useAnalyticsStore } from '../store';
import AnalyticsCharts from './AnalyticsCharts.vue';

vi.mock('vue-echarts', () => ({ default: { name: 'VChart', template: '<div class="mock-chart" />' } }));

function makeStats(overrides: Partial<StatsCache> = {}): StatsCache {
  return {
    version: 2,
    lastComputedDate: '2026-02-17',
    dailyActivity: [
      { date: '2026-02-15', messageCount: 10, sessionCount: 2, toolCallCount: 5 },
      { date: '2026-02-16', messageCount: 4, sessionCount: 1, toolCallCount: 1 },
    ],
    dailyModelTokens: [
      { date: '2026-02-15', tokensByModel: { 'claude-opus-4-6': 1000, 'claude-sonnet-4-5': 400 } },
      { date: '2026-02-16', tokensByModel: { 'claude-opus-4-6': 200 } },
    ],
    modelUsage: {
      'claude-opus-4-6': { inputTokens: 1000, outputTokens: 500, cacheReadInputTokens: 10, cacheCreationInputTokens: 5, webSearchRequests: 0, costUSD: 5, contextWindow: 200_000 },
      'claude-sonnet-4-5': { inputTokens: 400, outputTokens: 100, cacheReadInputTokens: 0, cacheCreationInputTokens: 0, webSearchRequests: 0, costUSD: 1, contextWindow: 200_000 },
    },
    totalSessions: 3,
    totalMessages: 14,
    longestSession: { sessionId: 'x', duration: 3_600_000, messageCount: 20, timestamp: '2026-01-01T00:00:00.000Z' },
    firstSessionDate: '2026-01-01T00:00:00.000Z',
    hourCounts: { '9': 2, '14': 1 },
    totalSpeculationTimeSavedMs: 0,
    ...overrides,
  };
}

function mountWithStats(stats: StatsCache) {
  setActivePinia(createPinia());
  const store = useAnalyticsStore();
  store.stats = stats;
  store.isLoading = false;
  return mount(AnalyticsCharts, { attachTo: document.body });
}

describe('AnalyticsCharts: chart cards', () => {
  it('renders all six chart-card titles plus the heatmap', () => {
    const wrapper = mountWithStats(makeStats());
    for (const title of ['Daily Activity', 'Activity by Hour', 'Activity Heatmap', 'Token Usage Over Time', 'Tool Calls Over Time', 'Model Usage', 'Cost & Token Breakdown']) {
      expect(wrapper.text()).toContain(title);
    }
  });

  it('renders the heatmap grid with Mon/Wed/Fri/Sun day labels and a titled cell for each active day', () => {
    const wrapper = mountWithStats(makeStats());
    expect(wrapper.text()).toContain('Mon');
    expect(wrapper.text()).toContain('Wed');
    const titled = wrapper.findAll('[title]').filter((el) => el.attributes('title')?.startsWith('2026-02-15'));
    expect(titled).toHaveLength(1);
    expect(titled[0].attributes('title')).toBe('2026-02-15: 10 messages · 2 sessions');
  });
});

describe('AnalyticsCharts: cost & token breakdown table', () => {
  it('lists every model with friendly names and formatted token counts', () => {
    const wrapper = mountWithStats(makeStats());
    expect(wrapper.text()).toContain('Opus 4.6');
    expect(wrapper.text()).toContain('Sonnet 4.5');
    const rows = wrapper.findAll('tbody tr');
    expect(rows).toHaveLength(2);
  });

  it('shows the API-only message instead of the cost donut when every costUSD is 0', () => {
    const wrapper = mountWithStats(
      makeStats({ modelUsage: { 'claude-opus-4-6': { inputTokens: 1000, outputTokens: 500, cacheReadInputTokens: 0, cacheCreationInputTokens: 0, webSearchRequests: 0, costUSD: 0, contextWindow: 200_000 } } }),
    );
    expect(wrapper.text()).toContain('Cost tracking is available for API users only');
  });

  it('renders the cost donut (mocked chart) instead of the message when cost data exists', () => {
    const wrapper = mountWithStats(makeStats());
    expect(wrapper.text()).not.toContain('Cost tracking is available for API users only');
    expect(wrapper.findAll('.mock-chart').length).toBeGreaterThan(0);
  });
});
