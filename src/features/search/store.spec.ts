// Carried over from docs/spec/search-overlay.md's own citations (M3-01). Router/session-store
// navigation itself is tested in SearchOverlay.spec.ts (store.ts never calls useRouter() -- see
// that file's own header comment); this file covers the store's own data/debounce/resolution
// logic in isolation.
import { mockIPC } from '@tauri-apps/api/mocks';
import { flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs, RecentSession, SearchResults, SessionSummary, SidebarCounts } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useSearchStore } from './store';

// One flag true, one false: proves the gate works in both directions in the same run, without
// needing to re-mock mid-file (vi.mock is hoisted once per file).
vi.mock('@/features/tray', () => ({ available: true, label: 'Tray' }));
vi.mock('@/features/config/mcp', () => ({ available: false, label: 'MCP Servers' }));

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

function zeroCounts(): SidebarCounts {
  return {
    todos: 0,
    history: 0,
    commands: 0,
    skills: 0,
    plans: 0,
    memory: 0,
    mcpServers: 0,
    models: 0,
    hooks: 0,
    subAgents: 0,
    plugins: 0,
    outputStyles: 0,
  };
}

function makeSessionSummary(overrides: Partial<SessionSummary> = {}): SessionSummary {
  return {
    id: 's1',
    projectId: 'proj',
    title: 'a session',
    preview: null,
    startedAt: '2026-01-01T00:00:00Z',
    endedAt: null,
    model: null,
    tokens: { input: 0, output: 0, cacheRead: 0, cacheCreation: 0 },
    turnCount: 0,
    isSidechain: false,
    parentSessionId: null,
    agentType: null,
    agentDescription: null,
    hasDebugLog: false,
    hasFileHistory: false,
    hasTodos: false,
    ...overrides,
  };
}

function makeRecent(overrides: Partial<RecentSession> = {}): RecentSession {
  return { projectId: 'proj', projectName: 'My Project', session: makeSessionSummary(), ...overrides };
}

function mockSearchIPC(options: { counts?: SidebarCounts; recent?: RecentSession[]; search?: (query: string, mode: string) => SearchResults } = {}) {
  mockIPC((cmd, args) => {
    if (cmd === 'get_sidebar_counts') return options.counts ?? zeroCounts();
    if (cmd === 'list_recent_sessions') return options.recent ?? [];
    if (cmd === 'search') {
      const a = args as { query: string; mode: string };
      return options.search ? options.search(a.query, a.mode) : { groups: [] };
    }
    if (cmd === 'update_prefs') return usePrefsStore().prefs;
    throw new Error(`unexpected command: ${cmd}`);
  });
}

beforeEach(() => {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('search store: empty-query state (§3)', () => {
  it('emptyScreens excludes sessions and has the 13 remaining NavigationItem entries', () => {
    const store = useSearchStore();
    expect(store.emptyScreens.some((i) => i.id === 'sessions')).toBe(false);
    expect(store.emptyScreens.length).toBe(13);
  });

  it('open() loads sidebar counts and the 5 most recent sessions once', async () => {
    mockSearchIPC({ counts: { ...zeroCounts(), todos: 3 }, recent: [makeRecent()] });
    const store = useSearchStore();

    await store.open();

    expect(store.isOpen).toBe(true);
    expect(store.sidebarCounts?.todos).toBe(3);
    expect(store.recentSessions.length).toBe(1);
  });
});

describe('search store: debounce and staleness (§8)', () => {
  it('does not update debouncedQuery (or search) until 200ms after the last keystroke', async () => {
    mockSearchIPC();
    const store = useSearchStore();

    store.query = 'ss';
    await vi.advanceTimersByTimeAsync(100);
    store.query = 'ssh'; // a second keystroke resets the window
    await vi.advanceTimersByTimeAsync(100);
    expect(store.debouncedQuery).toBe('');

    await vi.advanceTimersByTimeAsync(100);
    expect(store.debouncedQuery).toBe('ssh');
  });

  it('resets selectedIndex to 0 on every query change', async () => {
    const store = useSearchStore();
    store.selectedIndex = 4;
    store.query = 'x';
    await flushPromises(); // the query watcher runs on the next tick, not synchronously
    expect(store.selectedIndex).toBe(0);
  });

  it('a stale in-flight search never overwrites a newer one that resolved first', async () => {
    let resolveFirst!: (r: SearchResults) => void;
    const first = new Promise<SearchResults>((resolve) => (resolveFirst = resolve));
    const store = useSearchStore();
    mockIPC((cmd, args) => {
      if (cmd === 'get_sidebar_counts') return zeroCounts();
      if (cmd === 'list_recent_sessions') return [];
      if (cmd === 'search') {
        const a = args as { query: string };
        return a.query === 'first' ? first : { groups: [{ category: 'todos', hits: [{ title: 'second', subtitle: null, highlights: [], action: { kind: 'navigateTo', screen: 'todos' } }] }] };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    store.query = 'first';
    await vi.advanceTimersByTimeAsync(200);
    store.query = 'second';
    await vi.advanceTimersByTimeAsync(200);
    await Promise.resolve(); // let the second (already-resolved) search's .then settle
    resolveFirst({ groups: [{ category: 'todos', hits: [{ title: 'stale', subtitle: null, highlights: [], action: { kind: 'navigateTo', screen: 'todos' } }] }] });
    await Promise.resolve();
    await Promise.resolve();

    expect(store.flatResults.map((h) => h.title)).toEqual(['second']);
  });

  it('a mode switch re-searches immediately, with no debounce wait', async () => {
    let searchCalls = 0;
    mockIPC((cmd) => {
      if (cmd === 'get_sidebar_counts') return zeroCounts();
      if (cmd === 'list_recent_sessions') return [];
      if (cmd === 'update_prefs') return { ...usePrefsStore().prefs!, globalSearchMode: 'exact' };
      if (cmd === 'search') {
        searchCalls += 1;
        return { groups: [] };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useSearchStore();
    store.query = 'x';
    await vi.advanceTimersByTimeAsync(200);
    const searchesBefore = searchCalls;

    store.setMode('exact');
    await flushPromises();

    expect(store.mode).toBe('exact');
    expect(searchCalls).toBeGreaterThan(searchesBefore);
  });
});

describe('search store: quick actions (§4) -- gated by each feature\'s own available flag', () => {
  it('shows a quick action only when its owning feature reports available: true', () => {
    const store = useSearchStore();
    store.query = 'toggle menu bar icon';
    expect(store.quickActions.map((a) => a.label)).toEqual(['Toggle Menu Bar Icon']);
  });

  it('never shows a quick action whose owning feature reports available: false', () => {
    const store = useSearchStore();
    store.query = 'add mcp server';
    expect(store.quickActions).toEqual([]);
  });

  it('shows nothing for an empty query', () => {
    const store = useSearchStore();
    store.query = '';
    expect(store.quickActions).toEqual([]);
  });

  it('activating the available quick action flips showMenuBarIcon and does not navigate', () => {
    mockSearchIPC();
    const store = useSearchStore();
    usePrefsStore().prefs = defaultPrefs({ showMenuBarIcon: true });

    const result = store.activateQuickAction({ label: 'Toggle Menu Bar Icon', subtitle: '', icon: '', group: 'commands' });

    expect(result).toEqual({ kind: 'none' });
  });
});

describe('search store: activation resolution (§5)', () => {
  it('openSession resolves to a session activation', () => {
    const store = useSearchStore();
    store.query = 'x'; // activate() only reads groups/flatResults once the query is non-empty
    store.groups = [
      { category: 'sessions', hits: [{ title: 'S', subtitle: null, highlights: [], action: { kind: 'openSession', projectId: 'p1', sessionId: 's1' } }] },
    ];
    expect(store.activate()).toEqual({ kind: 'session', projectId: 'p1', sessionId: 's1' });
  });

  it('openDebugLog resolves to a session activation with debugLog: true', () => {
    const store = useSearchStore();
    store.query = 'x';
    store.groups = [
      { category: 'debugLogs', hits: [{ title: 'D', subtitle: null, highlights: [], action: { kind: 'openDebugLog', projectId: 'p1', sessionId: 's1' } }] },
    ];
    expect(store.activate()).toEqual({ kind: 'session', projectId: 'p1', sessionId: 's1', debugLog: true });
  });

  it('navigateTo resolves to the matching NavigationItem route', () => {
    const store = useSearchStore();
    store.query = 'x';
    store.groups = [{ category: 'todos', hits: [{ title: 'T', subtitle: null, highlights: [], action: { kind: 'navigateTo', screen: 'todos' } }] }];
    expect(store.activate()).toEqual({ kind: 'route', to: { name: 'todos' } });
  });

  it('openDetail with a generic-config ref resolves to the config-detail route', () => {
    const store = useSearchStore();
    store.query = 'x';
    store.groups = [
      {
        category: 'commands',
        hits: [
          {
            title: 'C',
            subtitle: null,
            highlights: [],
            action: { kind: 'openDetail', screen: 'commands', ref: { kind: 'command', project: null, slug: 'deploy' }, memoryFile: null },
          },
        ],
      },
    ];
    expect(store.activate()).toEqual({ kind: 'route', to: { name: 'config-detail', params: { kind: 'command', scope: 'global', slug: 'deploy' } } });
  });

  it('openDetail with a project-scoped ref resolves scope: "project"', () => {
    const store = useSearchStore();
    store.query = 'x';
    store.groups = [
      {
        category: 'skills',
        hits: [
          {
            title: 'S',
            subtitle: null,
            highlights: [],
            action: {
              kind: 'openDetail',
              screen: 'skills',
              ref: { kind: 'skill', project: { kind: 'project', projectId: 'proj' }, slug: 'reviewer' },
              memoryFile: null,
            },
          },
        ],
      },
    ];
    expect(store.activate()).toEqual({ kind: 'route', to: { name: 'config-detail', params: { kind: 'skill', scope: 'project', slug: 'reviewer' } } });
  });

  it('openDetail with no ref (e.g. Plans/Memory) degrades to a screen-level navigation', () => {
    const store = useSearchStore();
    store.query = 'x';
    store.groups = [
      { category: 'plans', hits: [{ title: 'P', subtitle: null, highlights: [], action: { kind: 'openDetail', screen: 'plans', ref: null, memoryFile: null } }] },
    ];
    expect(store.activate()).toEqual({ kind: 'route', to: { name: 'plans' } });
  });

  it('an out-of-range selectedIndex is ignored (§6: "otherwise ignored")', () => {
    const store = useSearchStore();
    store.query = 'x';
    store.groups = [];
    store.selectedIndex = 0;
    expect(store.activate()).toEqual({ kind: 'none' });
  });

  it('empty-query Return on a SCREENS row routes to that screen', () => {
    const store = useSearchStore();
    store.selectedIndex = 0;
    expect(store.activate()).toEqual({ kind: 'route', to: store.emptyScreens[0].to });
  });

  it('empty-query Return on a RECENT row opens that session', () => {
    const store = useSearchStore();
    store.recentSessions = [makeRecent({ projectId: 'p9', session: makeSessionSummary({ id: 's9' }) })];
    store.selectedIndex = store.emptyScreens.length;
    expect(store.activate()).toEqual({ kind: 'session', projectId: 'p9', sessionId: 's9' });
  });
});

describe('search store: keyboard selection bounds (§6)', () => {
  it('clamps moveSelection at 0 and at the last row, for both empty and search states', () => {
    const store = useSearchStore();
    store.moveSelection(-1);
    expect(store.selectedIndex).toBe(0);

    const max = store.emptyScreens.length + store.recentSessions.length - 1;
    store.selectedIndex = max;
    store.moveSelection(1);
    expect(store.selectedIndex).toBe(max);
  });
});
