// M3-58. AC: "Search calls `get_tray_snapshot(query)`; selecting a session calls `open_in_main_window`."
// The rest is what makes the window usable: it is hidden and shown rather than closed, so it must
// refresh when shown, and a slow search must never overwrite a newer one.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { NavTarget, Prefs, RecentSession, TraySnapshot } from '@/ipc/bindings';
import TrayView from './TrayView.vue';

const HOUR = 3_600_000;

function recent(id: string, title: string, project = 'demo-app', agoMs = HOUR): RecentSession {
  return {
    projectId: `p-${project}`,
    projectName: project,
    session: {
      id, projectId: `p-${project}`, title, preview: null, startedAt: new Date(Date.now() - agoMs).toISOString(), endedAt: null, model: null,
      tokens: { input: 0, output: 0, cacheRead: 0, cacheCreation: 0 }, turnCount: 1, isSidechain: false, parentSessionId: null,
      agentType: null, agentDescription: null, hasDebugLog: false, hasFileHistory: false, hasTodos: false,
    },
  };
}

function snapshot(overrides: Partial<TraySnapshot> = {}): TraySnapshot {
  return { totalSessions: 12, totalMessages: 340, todayMessages: 5, recent: [recent('s1', 'Fix the parser')], ...overrides };
}

function prefs(colorTheme: Prefs['colorTheme'] = 'default'): Prefs {
  return {
    schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme, fontScale: 1, sessionLayout: 'grid',
    configProjectPath: null, configLayouts: {}, showAgentSessions: false, textEditor: 'vscode', preferredTerminal: 'terminal',
    openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true, autoExpandBlocks: true,
    parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [],
  };
}

const failure = (message: string) => ({ kind: 'busy', message, path: null });

interface Ipc {
  snapshots: (query: string | null) => TraySnapshot | Promise<TraySnapshot>;
  opened: NavTarget[];
  queries: (string | null)[];
  prefsLoads: number;
  openFails?: string;
  prefsFor?: () => Prefs;
}

let mounted: ReturnType<typeof mount>[] = [];

function setUp(overrides: Partial<Ipc> = {}) {
  setActivePinia(createPinia());
  const ipc: Ipc = { snapshots: () => snapshot(), opened: [], queries: [], prefsLoads: 0, ...overrides };
  mockIPC((cmd, args) => {
    switch (cmd) {
      case 'get_tray_snapshot': {
        const query = (args as { query: string | null }).query;
        ipc.queries.push(query);
        return ipc.snapshots(query);
      }
      case 'open_in_main_window':
        ipc.opened.push((args as { target: NavTarget }).target);
        if (ipc.openFails) throw failure(ipc.openFails);
        return null;
      case 'get_prefs':
        ipc.prefsLoads++;
        return (ipc.prefsFor ?? prefs)();
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(TrayView, { attachTo: document.body });
  mounted.push(wrapper);
  return { wrapper, ipc };
}

const rows = (w: ReturnType<typeof mount>) => w.findAll('li button');
const typeInto = async (w: ReturnType<typeof mount>, text: string) => w.find('input').setValue(text);

beforeEach(() => {
  document.body.innerHTML = '';
  delete document.documentElement.dataset.theme;
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

describe('TrayView: header stats', () => {
  it('shows sessions, messages and messages today, with a hint on each', async () => {
    const { wrapper } = setUp({ snapshots: () => snapshot({ totalSessions: 12, totalMessages: 340, todayMessages: 5 }) });
    await flushPromises();
    const header = wrapper.find('header');
    expect(header.text()).toContain('Heimdall');
    expect(header.find('[title="Total sessions"]').text()).toBe('12sessions');
    expect(header.find('[title="Total messages"]').text()).toBe('340messages');
    expect(header.find('[title="Messages today"]').text()).toBe('5today');
  });

  it.each([
    [999, '999'],
    [1000, '1.0K'],
    [12_345, '12.3K'],
    [1_000_000, '1.0M'],
    [2_500_000, '2.5M'],
  ])('writes %i as %s', async (n, text) => {
    const { wrapper } = setUp({ snapshots: () => snapshot({ totalMessages: n }) });
    await flushPromises();
    expect(wrapper.find('[title="Total messages"]').text()).toContain(text);
  });

  it('shows no stats, and says it is loading, until the first answer arrives', async () => {
    let answer: (s: TraySnapshot) => void = () => {};
    const { wrapper } = setUp({ snapshots: () => new Promise((resolve) => (answer = resolve)) });
    await flushPromises();
    expect(wrapper.find('[title="Total sessions"]').exists()).toBe(false);
    expect(wrapper.text()).toContain('Loading');
    expect(wrapper.text()).not.toContain('No recent sessions');
    answer(snapshot());
    await flushPromises();
    expect(wrapper.text()).not.toContain('Loading');
    expect(wrapper.find('[title="Total sessions"]').exists()).toBe(true);
  });
});

describe('TrayView: recent sessions', () => {
  it('lists each session with its title, project and how long ago it started', async () => {
    const { wrapper } = setUp({
      snapshots: () => snapshot({ recent: [recent('s1', 'Fix the parser', 'demo-app', HOUR), recent('s2', 'Add tray', 'heimdall', 3 * 24 * HOUR)] }),
    });
    await flushPromises();
    expect(rows(wrapper)).toHaveLength(2);
    expect(rows(wrapper)[0].text()).toContain('Fix the parser');
    expect(rows(wrapper)[0].text()).toContain('demo-app');
    expect(rows(wrapper)[0].text()).toContain('1 hour ago');
    expect(rows(wrapper)[1].text()).toContain('3 days ago');
    expect(rows(wrapper)[0].attributes('title')).toBe('Fix the parser');
  });

  it('shows every session it is given (the cap of 10 is the command\'s)', async () => {
    const ten = Array.from({ length: 10 }, (_, i) => recent(`s${i}`, `Session ${i}`));
    const { wrapper } = setUp({ snapshots: () => snapshot({ recent: ten }) });
    await flushPromises();
    expect(rows(wrapper)).toHaveLength(10);
  });

  it('says there are none, differently for an empty list and an empty search', async () => {
    const { wrapper } = setUp({ snapshots: () => snapshot({ recent: [] }) });
    await flushPromises();
    expect(wrapper.text()).toContain('No recent sessions');
    expect(wrapper.text()).not.toContain('No matching sessions');
    vi.useFakeTimers();
    await typeInto(wrapper, 'zzz');
    await vi.advanceTimersByTimeAsync(300);
    expect(wrapper.text()).toContain('No matching sessions');
  });
});

describe('TrayView: search (AC: calls get_tray_snapshot(query))', () => {
  it('asks for everything with a null query first, then for what was typed, after a short pause', async () => {
    vi.useFakeTimers();
    const { wrapper, ipc } = setUp();
    await vi.advanceTimersByTimeAsync(0);
    expect(ipc.queries).toEqual([null]);

    await typeInto(wrapper, 'pars');
    await vi.advanceTimersByTimeAsync(100);
    expect(ipc.queries).toEqual([null]); // still typing
    await vi.advanceTimersByTimeAsync(150);
    expect(ipc.queries).toEqual([null, 'pars']);
  });

  it('sends one query for a burst of keystrokes, not one per key', async () => {
    vi.useFakeTimers();
    const { wrapper, ipc } = setUp();
    await vi.advanceTimersByTimeAsync(0);
    for (const text of ['p', 'pa', 'par', 'pars']) {
      await typeInto(wrapper, text);
      await vi.advanceTimersByTimeAsync(50);
    }
    await vi.advanceTimersByTimeAsync(300);
    expect(ipc.queries).toEqual([null, 'pars']);
  });

  it('trims the query, and asks for everything again (null) when it is cleared or blank', async () => {
    vi.useFakeTimers();
    const { wrapper, ipc } = setUp();
    await vi.advanceTimersByTimeAsync(0);
    await typeInto(wrapper, '  pars  ');
    await vi.advanceTimersByTimeAsync(300);
    await typeInto(wrapper, '   ');
    await vi.advanceTimersByTimeAsync(300);
    expect(ipc.queries).toEqual([null, 'pars', null]);
  });

  it('shows what the search returned', async () => {
    vi.useFakeTimers();
    const { wrapper } = setUp({
      snapshots: (q) => snapshot({ recent: q === 'tray' ? [recent('s2', 'Add tray')] : [recent('s1', 'Fix the parser'), recent('s2', 'Add tray')] }),
    });
    await vi.advanceTimersByTimeAsync(0);
    expect(rows(wrapper)).toHaveLength(2);
    await typeInto(wrapper, 'tray');
    await vi.advanceTimersByTimeAsync(300);
    expect(rows(wrapper).map((r) => r.attributes('title'))).toEqual(['Add tray']);
  });

  it('keeps the newer search when an older one answers last', async () => {
    vi.useFakeTimers();
    const pending: Record<string, (s: TraySnapshot) => void> = {};
    const { wrapper } = setUp({
      snapshots: (q) => (q === null ? snapshot() : new Promise((resolve) => (pending[q] = resolve))),
    });
    await vi.advanceTimersByTimeAsync(0);
    await typeInto(wrapper, 'old');
    await vi.advanceTimersByTimeAsync(300);
    await typeInto(wrapper, 'new');
    await vi.advanceTimersByTimeAsync(300);

    pending.new(snapshot({ recent: [recent('s-new', 'The newer answer')] }));
    await vi.advanceTimersByTimeAsync(0);
    pending.old(snapshot({ recent: [recent('s-old', 'The older answer')] }));
    await vi.advanceTimersByTimeAsync(0);

    expect(rows(wrapper).map((r) => r.attributes('title'))).toEqual(['The newer answer']);
  });

  it('can be cleared with the field\'s own button, which asks for everything again', async () => {
    vi.useFakeTimers();
    const { wrapper, ipc } = setUp();
    await vi.advanceTimersByTimeAsync(0);
    await typeInto(wrapper, 'pars');
    await vi.advanceTimersByTimeAsync(300);
    await wrapper.find('button[aria-label="Clear filter"]').trigger('click');
    await vi.advanceTimersByTimeAsync(300);
    expect(ipc.queries).toEqual([null, 'pars', null]);
  });
});

describe('TrayView: opening in the main window (AC: calls open_in_main_window)', () => {
  it('opens a session by its project and session id', async () => {
    const { wrapper, ipc } = setUp({ snapshots: () => snapshot({ recent: [recent('s-42', 'Fix the parser', 'demo-app')] }) });
    await flushPromises();
    await rows(wrapper)[0].trigger('click');
    await flushPromises();
    expect(ipc.opened).toEqual([{ kind: 'session', projectId: 'p-demo-app', sessionId: 's-42' }]);
  });

  it('opens the sessions screen from "Open Heimdall"', async () => {
    const { wrapper, ipc } = setUp();
    await flushPromises();
    await wrapper.find('button[title="Open the main window"]').trigger('click');
    await flushPromises();
    expect(ipc.opened).toEqual([{ kind: 'screen', screen: 'sessions' }]);
  });

  it('shows why it could not, and leaves the list as it was', async () => {
    const { wrapper } = setUp({ openFails: 'no main window' });
    await flushPromises();
    await rows(wrapper)[0].trigger('click');
    await flushPromises();
    expect(wrapper.find('[role="alert"]').text()).toBe('no main window');
    expect(rows(wrapper)).toHaveLength(1);
  });

  it('clears the message once something opens', async () => {
    const { wrapper, ipc } = setUp({ openFails: 'no main window' });
    await flushPromises();
    await rows(wrapper)[0].trigger('click');
    await flushPromises();
    ipc.openFails = undefined;
    await rows(wrapper)[0].trigger('click');
    await flushPromises();
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  });
});

describe('TrayView: errors', () => {
  it('shows the cause when the list cannot be loaded, rather than an empty list or a spinner', async () => {
    const { wrapper } = setUp({ snapshots: () => Promise.reject(failure('~/.claude is in use by another program')) });
    await flushPromises();
    expect(wrapper.find('[role="alert"]').text()).toBe('~/.claude is in use by another program');
    expect(wrapper.text()).not.toContain('Loading');
    expect(wrapper.text()).not.toContain('No recent sessions');
  });

  it('keeps showing the last list, and says why, when a refresh fails', async () => {
    let fail = false;
    const { wrapper } = setUp({ snapshots: () => (fail ? Promise.reject(failure('read failed')) : snapshot()) });
    await flushPromises();
    fail = true;
    window.dispatchEvent(new Event('focus'));
    await flushPromises();
    expect(rows(wrapper)).toHaveLength(1);
    expect(wrapper.find('[role="alert"]').text()).toBe('read failed');
  });

  it('clears the message when the next load works', async () => {
    let fail = true;
    const { wrapper } = setUp({ snapshots: () => (fail ? Promise.reject(failure('read failed')) : snapshot()) });
    await flushPromises();
    fail = false;
    window.dispatchEvent(new Event('focus'));
    await flushPromises();
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
    expect(rows(wrapper)).toHaveLength(1);
  });
});

describe('TrayView: the window is hidden and shown, not recreated', () => {
  it('reads the list again each time it is shown, so it is never a stale first load', async () => {
    let current = snapshot({ recent: [recent('s1', 'First load')] });
    const { wrapper, ipc } = setUp({ snapshots: () => current });
    await flushPromises();
    expect(rows(wrapper)[0].attributes('title')).toBe('First load');

    current = snapshot({ totalSessions: 99, recent: [recent('s2', 'Started while it was hidden')] });
    window.dispatchEvent(new Event('focus'));
    await flushPromises();

    expect(ipc.queries).toEqual([null, null]);
    expect(rows(wrapper)[0].attributes('title')).toBe('Started while it was hidden');
    expect(wrapper.find('[title="Total sessions"]').text()).toContain('99');
  });

  it('keeps what was typed, and searches for it again, when shown again', async () => {
    vi.useFakeTimers();
    const { wrapper, ipc } = setUp();
    await vi.advanceTimersByTimeAsync(0);
    await typeInto(wrapper, 'pars');
    await vi.advanceTimersByTimeAsync(300);
    window.dispatchEvent(new Event('focus'));
    await vi.advanceTimersByTimeAsync(0);
    expect(ipc.queries).toEqual([null, 'pars', 'pars']);
  });

  it('does one refresh when both focus and visibility fire for the same showing', async () => {
    const { ipc } = setUp();
    await flushPromises();
    window.dispatchEvent(new Event('focus'));
    document.dispatchEvent(new Event('visibilitychange'));
    await flushPromises();
    expect(ipc.queries).toEqual([null, null]);
    expect(ipc.prefsLoads).toBe(2);
  });

  it('does not refresh when it becomes hidden', async () => {
    const { ipc } = setUp();
    await flushPromises();
    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'hidden' });
    document.dispatchEvent(new Event('visibilitychange'));
    await flushPromises();
    delete (document as { visibilityState?: unknown }).visibilityState;
    expect(ipc.queries).toEqual([null]);
  });

  it('applies the saved theme on load, and the new one when shown after it was changed in the main window', async () => {
    let theme: Prefs['colorTheme'] = 'solarized';
    setUp({ prefsFor: () => prefs(theme) });
    await flushPromises();
    expect(document.documentElement.dataset.theme).toBe('solarized');

    theme = 'charcoal';
    window.dispatchEvent(new Event('focus'));
    await flushPromises();
    expect(document.documentElement.dataset.theme).toBe('charcoal');
  });

  it('stops listening once it is unmounted', async () => {
    const { wrapper, ipc } = setUp();
    await flushPromises();
    wrapper.unmount();
    window.dispatchEvent(new Event('focus'));
    document.dispatchEvent(new Event('visibilitychange'));
    await flushPromises();
    expect(ipc.queries).toEqual([null]);
  });
});
