// M3-03's own AC: "Layout choice persists per screen in configLayouts; detail renders through
// MarkdownView." The MarkdownView half has no store logic to test (PlanDetail.vue just passes
// `plan.body` straight through, already covered by MarkdownView's own tests); this file covers
// persistence (including the wholesale-replace gotcha configLayouts has, see below), loading,
// search and open/close.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { ConfigItemDetail, ConfigItemSummary, Prefs } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { usePlansStore } from './store';

function defaultPrefs(overrides: Partial<Prefs> = {}): Prefs {
  return {
    schemaVersion: 1,
    showMenuBarIcon: true,
    hasCompletedOnboarding: true,
    colorTheme: 'default',
    fontScale: 1.0,
    sessionLayout: 'grid',
    configProjectPath: null,
    configLayouts: {},
    showAgentSessions: false,
    textEditor: 'vscode',
    preferredTerminal: 'terminal',
    openTerminalOnBash: false,
    claudeCodePath: null,
    showAnimations: true,
    wrapCodeLines: true,
    autoExpandBlocks: true,
    parseMarkdownInResults: true,
    globalSearchMode: 'fuzzy',
    customMemoryFolders: [],
    ...overrides,
  };
}

function makeSummary(overrides: Partial<ConfigItemSummary> = {}): ConfigItemSummary {
  return { ref: { kind: 'plan', project: null, slug: 'refactor-auth-module' }, name: 'Refactor Auth Module', description: '', scope: 'global', meta: {}, ...overrides };
}

function makeDetail(overrides: Partial<ConfigItemDetail> = {}): ConfigItemDetail {
  return {
    ref: { kind: 'plan', project: null, slug: 'refactor-auth-module' },
    name: 'Refactor Auth Module',
    description: '',
    scope: 'global',
    meta: {},
    body: '# Plan\n\nDo the thing.',
    displayPath: '~/.claude/plans/refactor-auth-module.md',
    ...overrides,
  };
}

beforeEach(() => {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
});

describe('plans store: loading', () => {
  it('load() calls listConfigItems with kind "plan" and no project (global only)', async () => {
    mockIPC((cmd, args) => {
      expect(cmd).toBe('list_config_items');
      expect(args).toEqual({ kind: 'plan', project: null });
      return { value: [makeSummary()], skipped: 0 };
    });
    const store = usePlansStore();

    await store.load();

    expect(store.plans.length).toBe(1);
    expect(store.isLoading).toBe(false);
  });
});

describe('plans store: layout persistence (AC: "Layout choice persists per screen in configLayouts")', () => {
  it('defaults to grid when nothing has been saved yet', () => {
    const store = usePlansStore();
    expect(store.layout).toBe('grid');
  });

  it('reads the saved layout for the "plans" key specifically', () => {
    usePrefsStore().prefs = defaultPrefs({ configLayouts: { plans: 'list' } });
    const store = usePlansStore();
    expect(store.layout).toBe('list');
  });

  it('toggleLayout() flips grid <-> list and persists it', async () => {
    mockIPC((cmd, args) => {
      expect(cmd).toBe('update_prefs');
      return { ...usePrefsStore().prefs!, configLayouts: (args as { patch: { configLayouts: Record<string, string> } }).patch.configLayouts };
    });
    const store = usePlansStore();
    expect(store.layout).toBe('grid');

    await store.toggleLayout();

    expect(store.layout).toBe('list');
  });

  // The real regression this guards against: `update_prefs` replaces `configLayouts` wholesale
  // (prefs.rs's own apply_patch: `prefs.config_layouts = v`, not a per-key merge) -- naively
  // sending only `{ plans: 'list' }` would silently wipe every other screen's own saved layout.
  it('setLayout() preserves every other screen\'s own saved layout', async () => {
    usePrefsStore().prefs = defaultPrefs({ configLayouts: { sessions: 'list', hooks: 'grid' } });
    let sentPatch: unknown;
    mockIPC((_cmd, args) => {
      sentPatch = (args as { patch: { configLayouts: unknown } }).patch.configLayouts;
      return { ...usePrefsStore().prefs!, configLayouts: sentPatch };
    });
    const store = usePlansStore();

    await store.setLayout('list');

    expect(sentPatch).toEqual({ sessions: 'list', hooks: 'grid', plans: 'list' });
  });
});

describe('plans store: search', () => {
  it('filters by a fuzzy match on the plan name', async () => {
    mockIPC(() => ({
      value: [makeSummary({ ref: { kind: 'plan', project: null, slug: 'auth' }, name: 'Refactor Auth Module' }), makeSummary({ ref: { kind: 'plan', project: null, slug: 'ui' }, name: 'Redesign UI' })],
      skipped: 0,
    }));
    const store = usePlansStore();
    await store.load();

    store.query = 'auth';

    expect(store.filteredPlans.map((p) => p.ref.slug)).toEqual(['auth']);
  });

  it('an empty query returns every plan, unfiltered', async () => {
    mockIPC(() => ({ value: [makeSummary(), makeSummary({ ref: { kind: 'plan', project: null, slug: 'other' }, name: 'Other' })], skipped: 0 }));
    const store = usePlansStore();
    await store.load();

    expect(store.filteredPlans.length).toBe(2);
  });
});

describe('plans store: open/close (AC: "detail renders through MarkdownView")', () => {
  it('open() fetches the full detail (including body) via getConfigItem', async () => {
    mockIPC((cmd, args) => {
      expect(cmd).toBe('get_config_item');
      expect(args).toEqual({ configRef: { kind: 'plan', project: null, slug: 'refactor-auth-module' } });
      return makeDetail();
    });
    const store = usePlansStore();

    await store.open({ kind: 'plan', project: null, slug: 'refactor-auth-module' });

    expect(store.selected?.body).toBe('# Plan\n\nDo the thing.');
    expect(store.isLoadingDetail).toBe(false);
  });

  it('close() clears the selected detail, returning to the list', async () => {
    mockIPC(() => makeDetail());
    const store = usePlansStore();
    await store.open({ kind: 'plan', project: null, slug: 'refactor-auth-module' });

    store.close();

    expect(store.selected).toBeNull();
  });
});
