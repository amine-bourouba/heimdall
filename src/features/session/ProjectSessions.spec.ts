// AC: "Spec covers both layouts and sub-agent sessions."
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { Prefs, SessionSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import ProjectSessions from './ProjectSessions.vue';
import { useSessionStore, type ProjectWithSessions } from './store';

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
    showAgentSessions: true,
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

function makeSession(overrides: Partial<SessionSummary> = {}): SessionSummary {
  return {
    id: 's1',
    projectId: 'p1',
    title: 'a session',
    preview: null,
    startedAt: '2026-01-01T00:00:00Z',
    endedAt: null,
    model: null,
    tokens: { input: 0, output: 0, cacheRead: 0, cacheCreation: 0 },
    turnCount: 1,
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

function makeProject(sessions: SessionSummary[]): ProjectWithSessions {
  return { id: 'p1', name: 'alpha', path: '/p1', pathSource: 'sessionsIndex', sessionCount: sessions.length, lastActivity: null, hasMemory: false, sessions };
}

let mounted: ReturnType<typeof mount>[] = [];

function mountView(sessions: SessionSummary[], prefsOverrides: Partial<Prefs> = {}) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs(prefsOverrides);
  mockIPC((cmd, args) => {
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      const current = usePrefsStore().prefs!;
      usePrefsStore().prefs = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return usePrefsStore().prefs;
    }
    if (cmd === 'get_session') return { summary: sessions[0], messageCount: 0, context: null, toolNames: [], skipped: 0 };
    if (cmd === 'get_session_messages') return { start: 0, messages: [], total: 0 };
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(ProjectSessions, { props: { project: makeProject(sessions) }, attachTo: document.body });
  mounted.push(wrapper);
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('ProjectSessions: grid layout', () => {
  it('renders every session as a grid card when sessionLayout is grid', () => {
    const wrapper = mountView([makeSession({ id: 's1', title: 'first' }), makeSession({ id: 's2', title: 'second' })], { sessionLayout: 'grid' });
    expect(wrapper.find('.grid').exists()).toBe(true);
    expect(wrapper.text()).toContain('first');
    expect(wrapper.text()).toContain('second');
  });
});

describe('ProjectSessions: list layout', () => {
  it('renders sessions as a list when sessionLayout is list', () => {
    const wrapper = mountView([makeSession({ title: 'only one' })], { sessionLayout: 'list' });
    expect(wrapper.find('.grid').exists()).toBe(false);
    expect(wrapper.text()).toContain('only one');
  });

  it('the toggle button flips sessionLayout through a real update_prefs round trip', async () => {
    const wrapper = mountView([makeSession()], { sessionLayout: 'grid' });
    expect(wrapper.find('.grid').exists()).toBe(true);

    await wrapper.find('button[title="Toggle layout"]').trigger('click');
    await flushPromises();

    expect(usePrefsStore().prefs?.sessionLayout).toBe('list');
    expect(wrapper.find('.grid').exists()).toBe(false);
  });
});

describe('ProjectSessions: search filtering', () => {
  it('filters to sessions matching the query by title, preview or id', async () => {
    const wrapper = mountView([
      makeSession({ id: 's1', title: 'fix auth bug' }),
      makeSession({ id: 's2', title: 'unrelated', preview: null }),
    ]);

    await wrapper.find('input[aria-label="Find in Sessions"]').setValue('auth');
    await flushPromises();

    expect(wrapper.text()).toContain('fix auth bug');
    expect(wrapper.text()).not.toContain('unrelated');
  });

  it('shows the empty-search state when nothing matches', async () => {
    const wrapper = mountView([makeSession({ title: 'fix auth bug' })]);

    await wrapper.find('input[aria-label="Find in Sessions"]').setValue('zzz-no-match');
    await flushPromises();

    expect(wrapper.text()).toContain('No sessions match "zzz-no-match"');
  });
});

describe('ProjectSessions: sub-agent sessions', () => {
  it('a sidechain session shows the agent badge with its description', () => {
    const wrapper = mountView([makeSession({ isSidechain: true, agentDescription: 'Explore the codebase' })]);
    expect(wrapper.text()).toContain('Explore the codebase');
  });

  it('falls back to agentType, then "Agent", when no description is set', () => {
    const withType = mountView([makeSession({ isSidechain: true, agentType: 'plan', agentDescription: null })]);
    expect(withType.text()).toContain('plan');

    const withNeither = mountView([makeSession({ isSidechain: true, agentType: null, agentDescription: null })]);
    expect(withNeither.text()).toContain('Agent');
  });

  it('a non-sidechain session shows no agent badge at all', () => {
    const wrapper = mountView([makeSession({ isSidechain: false })]);
    expect(wrapper.text()).not.toContain('Agent');
  });
});

describe('ProjectSessions: selecting a session', () => {
  it('clicking a card selects it through the real session store', async () => {
    const wrapper = mountView([makeSession({ id: 's1', title: 'pick me' })]);

    await wrapper.find('button[title="pick me"]').trigger('click');
    await flushPromises();

    expect(useSessionStore().selectedSession?.id).toBe('s1');
  });
});
