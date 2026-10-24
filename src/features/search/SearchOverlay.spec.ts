// M3-01's own AC list, as UI-level integration coverage (store.spec.ts already covers the
// resolution/debounce logic in isolation -- this file checks that this component wires it to the
// real DOM/router correctly: opening via the ⌘K action, rendering, keyboard activation, and
// router navigation actually happening).
//
// Every query below goes through `document.body` (`body()`), not `wrapper.find()`: this
// component's root is a `<Teleport to="body">`, and Vue Test Utils does not search teleported
// content from the mounting wrapper -- confirmed directly (every `wrapper.find()` came back an
// empty DOMWrapper on a real run) and documented behaviour
// (https://test-utils.vuejs.org/guide/advanced/teleport.html).
import { defineComponent, h } from 'vue';
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs, RecentSession, SearchResults, SessionSummary, SidebarCounts } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useAppActions } from '@/composables/useAppActions';
import SearchOverlay from './SearchOverlay.vue';

vi.mock('@/features/tray', () => ({ available: false, label: 'Tray' }));
vi.mock('@/features/config/mcp', () => ({ available: false, label: 'MCP Servers' }));

function body() {
  return new DOMWrapper(document.body);
}

const Blank = defineComponent({ render: () => h('div') });

// Same reasoning as Sidebar.spec.ts's own makeRouter(): plain sync stub routes, not the app's
// real lazy () => import(...) pages, so router.push resolves within one flushPromises().
function makeRouter() {
  return createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/', redirect: '/sessions' },
      { path: '/sessions', name: 'sessions', component: Blank },
      { path: '/sessions/:projectId/:sessionId?', name: 'session-detail', component: Blank },
      { path: '/todos', name: 'todos', component: Blank },
      { path: '/analytics', name: 'analytics', component: Blank },
    ],
  });
}

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

let mounted: ReturnType<typeof mount>[] = [];

async function mountOverlay(options: { recent?: RecentSession[]; search?: SearchResults } = {}) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  const router = makeRouter();
  await router.push('/');

  mockIPC((cmd) => {
    if (cmd === 'get_sidebar_counts') return zeroCounts();
    if (cmd === 'list_recent_sessions') return options.recent ?? [];
    if (cmd === 'search') return options.search ?? { groups: [] };
    if (cmd === 'update_prefs') return { ...usePrefsStore().prefs!, globalSearchMode: 'exact' };
    throw new Error(`unexpected command: ${cmd}`);
  });

  const wrapper = mount(SearchOverlay, { attachTo: document.body, global: { plugins: [router] } });
  mounted.push(wrapper);
  await flushPromises();
  return { wrapper, router };
}

async function openOverlay() {
  useAppActions().dispatch('search');
  await flushPromises();
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

describe('SearchOverlay: opening (⌘K dispatches the "search" AppAction)', () => {
  it('is closed by default and opens when the "search" action is dispatched', async () => {
    await mountOverlay();
    expect(body().find('[role="dialog"]').exists()).toBe(false);

    await openOverlay();

    expect(body().find('[role="dialog"]').exists()).toBe(true);
  });

  it('focuses the search input on open', async () => {
    await mountOverlay();
    await openOverlay();

    expect(document.activeElement).toBe(body().find('input').element);
  });

  it('toggles closed on a second dispatch', async () => {
    await mountOverlay();
    await openOverlay();
    await openOverlay();

    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });
});

describe('SearchOverlay: empty-query state (§3)', () => {
  it('shows the SCREENS and RECENT sections', async () => {
    await mountOverlay({ recent: [{ projectId: 'proj', projectName: 'My Project', session: makeSessionSummary({ title: 'Recent one' }) }] });
    await openOverlay();

    expect(body().text()).toContain('SCREENS');
    expect(body().text()).toContain('TODOs'); // src/features/todos/index.ts's own real label casing
    expect(body().text()).toContain('RECENT');
    expect(body().text()).toContain('Recent one');
  });

  it('clicking a SCREENS row navigates there and closes the overlay', async () => {
    const { router } = await mountOverlay();
    await openOverlay();

    const row = body().findAll('[role="option"]').find((r) => r.text().includes('TODOs'))!;
    await row.trigger('click');
    await flushPromises();

    expect(router.currentRoute.value.name).toBe('todos');
    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });
});

describe('SearchOverlay: grouped results and highlighting (§2.4, §7)', () => {
  it('renders a category header, the hit, and the highlighted substring as a <mark>', async () => {
    await mountOverlay({
      search: { groups: [{ category: 'todos', hits: [{ title: 'write tests', subtitle: null, highlights: [[6, 11]], action: { kind: 'navigateTo', screen: 'todos' } }] }] },
    });
    await openOverlay();

    await body().find('input').setValue('tests');
    await vi.waitFor(() => expect(body().text()).toContain('write tests'));

    expect(body().text()).toContain('TODOS');
    const mark = body().find('mark');
    expect(mark.exists()).toBe(true);
    expect(mark.text()).toBe('tests');
  });
});

describe('SearchOverlay: keyboard (§6)', () => {
  it('ArrowDown then Enter activates the second SCREENS row and navigates there', async () => {
    const { router } = await mountOverlay();
    await openOverlay();

    const input = body().find('input');
    await input.trigger('keydown', { key: 'ArrowDown' });
    await input.trigger('keydown', { key: 'Enter' });
    await flushPromises();

    // emptyScreens[1] is the second NavigationItem entry once "sessions" is filtered out.
    expect(router.currentRoute.value.name).not.toBe('sessions');
    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });

  it('Escape closes the overlay', async () => {
    await mountOverlay();
    await openOverlay();

    await body().find('input').trigger('keydown', { key: 'Escape' });
    await flushPromises();

    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });

  it('clicking the scrim also closes it', async () => {
    await mountOverlay();
    await openOverlay();

    await body().find('.fixed.inset-0').trigger('click');
    await flushPromises();

    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });
});

describe('SearchOverlay: match mode toggle (§6, §9)', () => {
  it('clicking "Exact" persists globalSearchMode through prefsStore.update', async () => {
    await mountOverlay();
    await openOverlay();

    const exactButton = body().findAll('button').find((b) => b.text() === 'Exact')!;
    await exactButton.trigger('click');
    await flushPromises();

    expect(usePrefsStore().prefs?.globalSearchMode).toBe('exact');
  });
});
