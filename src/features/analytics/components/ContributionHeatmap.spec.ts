// M3-34's own AC: "Spec checks week alignment across a year boundary and cell intensity buckets."
// Neither was ever exercised before this task -- AnalyticsCharts.spec.ts (M3-32) only covered a
// single-month fixture.
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { describe, expect, it } from 'vitest';
import type { StatsCache } from '@/ipc/bindings';
import { useAnalyticsStore } from '../store';
import ContributionHeatmap from './ContributionHeatmap.vue';

function mountWithActivity(dailyActivity: StatsCache['dailyActivity']) {
  setActivePinia(createPinia());
  const store = useAnalyticsStore();
  store.stats = {
    version: 2,
    lastComputedDate: '2026-02-17',
    dailyActivity,
    dailyModelTokens: [],
    modelUsage: {},
    totalSessions: 0,
    totalMessages: 0,
    longestSession: undefined,
    firstSessionDate: undefined,
    hourCounts: {},
    totalSpeculationTimeSavedMs: 0,
  };
  store.isLoading = false;
  return mount(ContributionHeatmap, { attachTo: document.body });
}

describe('ContributionHeatmap: week alignment across a year boundary', () => {
  it('keeps every real day in its own correctly-positioned Monday-first cell spanning Dec 29 -> Jan 4', () => {
    // 2025-12-29 is a Monday; 2026-01-04 is the following Sunday -- one full week, split across
    // the year boundary, every day present.
    const dailyActivity = [
      { date: '2025-12-29', messageCount: 1, sessionCount: 1, toolCallCount: 0 },
      { date: '2025-12-30', messageCount: 2, sessionCount: 1, toolCallCount: 0 },
      { date: '2025-12-31', messageCount: 3, sessionCount: 1, toolCallCount: 0 },
      { date: '2026-01-01', messageCount: 4, sessionCount: 1, toolCallCount: 0 },
      { date: '2026-01-02', messageCount: 5, sessionCount: 1, toolCallCount: 0 },
      { date: '2026-01-03', messageCount: 6, sessionCount: 1, toolCallCount: 0 },
      { date: '2026-01-04', messageCount: 7, sessionCount: 1, toolCallCount: 0 },
    ];
    const wrapper = mountWithActivity(dailyActivity);
    const cells = wrapper.findAll('[role="gridcell"]');
    expect(cells).toHaveLength(7);
    // Monday-first order: row 0 = Mon (Dec 29) ... row 6 = Sun (Jan 4), all in week-column 0.
    const expectedOrder = ['2025-12-29', '2025-12-30', '2025-12-31', '2026-01-01', '2026-01-02', '2026-01-03', '2026-01-04'];
    expect(cells.map((c) => c.attributes('aria-label')?.split(':')[0])).toEqual(expectedOrder);

    const rows = wrapper.findAll('[role="row"]');
    expect(rows).toHaveLength(1); // one Monday-to-Sunday week, so exactly one week-column
  });

  it('places a lone Jan 1 (a Thursday) at day-slot index 3, not slot 0, confirming the Monday remap carries across the boundary', () => {
    const wrapper = mountWithActivity([{ date: '2026-01-01', messageCount: 1, sessionCount: 1, toolCallCount: 0 }]);
    const row = wrapper.find('[role="row"]');
    expect(row.exists()).toBe(true);
    const slots = row.findAll(':scope > div'); // all 7 day-of-week slots, hidden padding included
    expect(slots).toHaveLength(7);
    expect(slots.map((s) => s.attributes('role'))).toEqual([undefined, undefined, undefined, 'gridcell', undefined, undefined, undefined]);
    expect(slots[3].attributes('aria-label')).toContain('2026-01-01');
  });
});

describe('ContributionHeatmap: cell intensity buckets', () => {
  it('buckets messages/max ratio into the 4 documented bands', () => {
    // max = 100 (the last entry): ratios are 0.1 (<0.25), 0.3 (<0.5), 0.6 (<0.75), 1.0 (else).
    const dailyActivity = [
      { date: '2026-01-05', messageCount: 10, sessionCount: 1, toolCallCount: 0 }, // Mon
      { date: '2026-01-06', messageCount: 30, sessionCount: 1, toolCallCount: 0 }, // Tue
      { date: '2026-01-07', messageCount: 60, sessionCount: 1, toolCallCount: 0 }, // Wed
      { date: '2026-01-08', messageCount: 100, sessionCount: 1, toolCallCount: 0 }, // Thu
    ];
    const wrapper = mountWithActivity(dailyActivity);
    const cells = wrapper.findAll('[role="gridcell"]');
    expect(cells).toHaveLength(4);
    expect(cells[0].classes()).toContain('bg-(--accent)/20');
    expect(cells[1].classes()).toContain('bg-(--accent)/40');
    expect(cells[2].classes()).toContain('bg-(--accent)/65');
    expect(cells[3].classes()).toContain('bg-(--accent)');
    expect(cells[3].classes()).not.toContain('bg-(--accent)/20');
  });

  it('gives a zero-message day the plain card background, not an accent band', () => {
    const wrapper = mountWithActivity([{ date: '2026-01-05', messageCount: 0, sessionCount: 0, toolCallCount: 0 }]);
    const cells = wrapper.findAll('[role="gridcell"]');
    expect(cells).toHaveLength(1);
    expect(cells[0].classes()).toContain('bg-(--bg-card)');
  });
});

describe('ContributionHeatmap: ARIA grid semantics', () => {
  it('marks padding cells outside the real date range as aria-hidden, not empty gridcells', () => {
    // 2026-01-07 is a Wednesday -- a single-day range still pads out the rest of that week's row.
    const wrapper = mountWithActivity([{ date: '2026-01-07', messageCount: 5, sessionCount: 1, toolCallCount: 0 }]);
    expect(wrapper.find('[role="grid"]').exists()).toBe(true);
    expect(wrapper.findAll('[role="gridcell"]')).toHaveLength(1);
    expect(wrapper.findAll('[aria-hidden="true"]').length).toBeGreaterThan(0);
  });

  it('shows nothing (no grid at all) when there is no activity data', () => {
    const wrapper = mountWithActivity([]);
    expect(wrapper.find('[role="grid"]').exists()).toBe(false);
  });
});
