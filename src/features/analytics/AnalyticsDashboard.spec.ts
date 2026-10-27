// M3-32's own AC: "The CSV menu offers the four kinds and calls export_analytics_csv." / "When
// all costUSD values are 0, cost areas show the API-only message."
//
// `vue-echarts` is stubbed out: it needs a real canvas 2D context to initialise, which happy-dom
// doesn't provide -- every other component here (stat cards, the CSV menu, the no-data/loading
// states) is tested against the real component, only the chart canvas itself is replaced with a
// trivial marker div, the standard way to unit-test an ECharts-wrapping component.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StatsCache } from '@/ipc/bindings';
import AnalyticsDashboard from './AnalyticsDashboard.vue';

// AnalyticsCharts.vue is only ever reached through `defineAsyncComponent` (M3-32's bundle-size
// fix) and is covered by its own direct-mount spec (AnalyticsCharts.spec.ts) -- stubbed here, by
// name via VTU's own `global.stubs` (see `mountDashboard` below), so this file's tests (stat
// cards/CSV menu/loading/no-data states) never need the async boundary to settle at all: a real
// reproduction showed the dynamic `import()` that loader fires never resolves under Vitest's
// happy-dom + Vite SSR module runner when driven through `mount()` (no error, no rejection, even
// though the same import resolves immediately when awaited directly or mounted synchronously) --
// a tooling quirk, not an app bug. `vi.mock`-ing the module directly (tried first) replaces it
// with a plain object that VTU's own internal vnode-type introspection (which runs for every
// child, checking several Vue-internal markers like `__v_isVNode`/`__isTeleport` to decide how to
// handle it) then probes -- CI's Linux/coverage run hit a marker the Windows/no-coverage run
// never did, confirming this is whack-a-mole, not a real fix. Stubbing by name intercepts the
// component before `defineAsyncComponent` ever resolves it, so none of this applies.
vi.mock('vue-echarts', () => ({ default: { name: 'VChart', template: '<div class="mock-chart" />' } }));

function makeStats(overrides: Partial<StatsCache> = {}): StatsCache {
  return {
    version: 2,
    lastComputedDate: '2026-02-17',
    dailyActivity: [{ date: '2026-02-15', messageCount: 10, sessionCount: 2, toolCallCount: 5 }],
    dailyModelTokens: [{ date: '2026-02-15', tokensByModel: { 'claude-opus-4-6': 1000 } }],
    modelUsage: { 'claude-opus-4-6': { inputTokens: 1000, outputTokens: 500, cacheReadInputTokens: 0, cacheCreationInputTokens: 0, webSearchRequests: 0, costUSD: 5, contextWindow: 200_000 } },
    totalSessions: 3,
    totalMessages: 42,
    longestSession: { sessionId: 'x', duration: 3_600_000, messageCount: 20, timestamp: '2026-01-01T00:00:00.000Z' },
    firstSessionDate: '2026-01-01T00:00:00.000Z',
    hourCounts: { '9': 2 },
    totalSpeculationTimeSavedMs: 0,
    ...overrides,
  };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountDashboard(stats: StatsCache | null, extraHandler?: (cmd: string, args: unknown) => unknown) {
  setActivePinia(createPinia());
  mockIPC((cmd, args) => {
    if (cmd === 'get_stats') {
      if (stats === null) throw { kind: 'notFound', message: 'no stats', path: null };
      return { stats, source: 'cache' };
    }
    if (cmd === 'get_prefs') {
      return {
        schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1.0, sessionLayout: 'grid', configProjectPath: null, configLayouts: {},
        showAgentSessions: false, textEditor: 'vscode', preferredTerminal: 'terminal', openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true,
        autoExpandBlocks: true, parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [],
      };
    }
    if (extraHandler) {
      const result = extraHandler(cmd, args);
      if (result !== undefined) return result;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(AnalyticsDashboard, { attachTo: document.body, global: { stubs: { AnalyticsCharts: true } } });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('AnalyticsDashboard: loading and empty states', () => {
  it('shows the shimmer while get_stats is pending, then real content once it resolves', async () => {
    const wrapper = await mountDashboard(makeStats());
    expect(wrapper.text()).toContain('Total Sessions');
    expect(wrapper.text()).toContain('42'); // totalMessages, under 1000 so formatLargeNumber leaves it as plain digits
  });

  it('shows the no-data notice when get_stats returns not found', async () => {
    const wrapper = await mountDashboard(null);
    expect(wrapper.text()).toContain('No session analytics yet');
  });
});

describe('AnalyticsDashboard: stat cards', () => {
  it('renders all 8 cards with the real totals', async () => {
    const wrapper = await mountDashboard(makeStats());
    expect(wrapper.text()).toContain('Total Sessions');
    expect(wrapper.text()).toContain('Total Messages');
    expect(wrapper.text()).toContain('Longest Session');
    expect(wrapper.text()).toContain('First Session');
    expect(wrapper.text()).toContain('Total Cost');
    expect(wrapper.text()).toContain('Total Tokens');
    expect(wrapper.text()).toContain('Tool Calls');
    expect(wrapper.text()).toContain('Time Saved');
  });
});

describe('AnalyticsDashboard: API-only cost message (AC2)', () => {
  // The equivalent chart-area message ("Cost tracking is available for API users only") lives in
  // AnalyticsCharts.vue, stubbed out above -- covered by AnalyticsCharts.spec.ts instead.
  it('shows the API-only message on the Total Cost stat card when every costUSD is 0', async () => {
    const wrapper = await mountDashboard(
      makeStats({ modelUsage: { 'claude-opus-4-6': { inputTokens: 1000, outputTokens: 500, cacheReadInputTokens: 0, cacheCreationInputTokens: 0, webSearchRequests: 0, costUSD: 0, contextWindow: 200_000 } } }),
    );
    expect(wrapper.text()).toContain('included in subscription');
  });

  it('shows a real cost figure, not the API-only message, when costUSD is non-zero', async () => {
    const wrapper = await mountDashboard(makeStats());
    expect(wrapper.text()).not.toContain('included in subscription');
    expect(wrapper.text()).toContain('$5.00');
  });
});

describe('AnalyticsDashboard: CSV export menu (AC1)', () => {
  it('offers all four kinds and calls export_analytics_csv with the right kind for each', async () => {
    const calledKinds: unknown[] = [];
    const wrapper = await mountDashboard(makeStats(), (cmd, args) => {
      if (cmd === 'export_analytics_csv') {
        calledKinds.push((args as { kind: string }).kind);
        return 'written';
      }
    });

    await wrapper.find('button[title="Export CSV"]').trigger('click');
    const menu = wrapper.find('[role="menu"][aria-label="Export CSV"]');
    const items = menu.findAll('button');
    expect(items.map((b) => b.text())).toEqual(['Daily Activity', 'Model Usage', 'Daily Tokens', 'Hourly Activity']);
    // Close it again: the trigger button is a plain toggle, so leaving the menu open here would
    // make the loop's own first click close it instead of reopening it.
    await wrapper.find('button[title="Export CSV"]').trigger('click');

    for (const [index, kind] of ['dailyActivity', 'modelUsage', 'dailyTokens', 'hourlyActivity'].entries()) {
      await wrapper.find('button[title="Export CSV"]').trigger('click');
      await wrapper.findAll('[role="menu"][aria-label="Export CSV"] button')[index].trigger('click');
      await flushPromises();
      expect(calledKinds.at(-1)).toBe(kind);
    }
  });
});
