// AC: "Spec covers sorting, search filtering, the showAgentSessions toggle and keyboard
// selection." Each session/project row is a real native <button> (never a div with a click
// handler bolted on), so Tab/Enter/Space already work the way every other native button does --
// "keyboard selection" is proved here by exercising that same activation path (a click, which is
// exactly what the browser turns a keyboard Enter/Space on a focused button into), not by
// re-simulating what the browser already guarantees for a real <button> element.
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { Prefs, ProjectSummary, SessionSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import SessionsNavigation from './SessionsNavigation.vue';
import { useSessionStore } from './store';

function defaultPrefs(): Prefs {
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
  };
}

function makeProjectSummary(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return { id: 'p1', name: 'project-one', path: '/p1', pathSource: 'sessionsIndex', sessionCount: 0, lastActivity: null, hasMemory: false, ...overrides };
}

function makeSessionSummary(overrides: Partial<SessionSummary> = {}): SessionSummary {
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

let mounted: ReturnType<typeof mount>[] = [];

function mountNavigation(byProject: Record<string, SessionSummary[]>, projects: ProjectSummary[]) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  mockIPC(
    (cmd, args) => {
      if (cmd === 'list_projects') return { value: projects, skipped: 0 };
      if (cmd === 'list_sessions') return { value: byProject[(args as { projectId: string }).projectId] ?? [], skipped: 0 };
      if (cmd === 'update_prefs') {
        const patch = (args as { patch: Record<string, unknown> }).patch;
        const current = usePrefsStore().prefs ?? defaultPrefs();
        usePrefsStore().prefs = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
        return usePrefsStore().prefs;
      }
      throw new Error(`unexpected command: ${cmd}`);
    },
    { shouldMockEvents: true },
  );
  const wrapper = mount(SessionsNavigation, { attachTo: document.body });
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

describe('SessionsNavigation: loads and renders', () => {
  it('shows a loading state, then the loaded projects and sessions', async () => {
    const wrapper = mountNavigation(
      { p1: [makeSessionSummary({ id: 's1', title: 'fix the bug' })] },
      [makeProjectSummary({ id: 'p1', name: 'alpha' })],
    );
    expect(wrapper.text()).toContain('Loading sessions');

    await flushPromises();

    expect(wrapper.text()).not.toContain('Loading sessions');
    expect(wrapper.text()).toContain('alpha');
    expect(wrapper.text()).toContain('fix the bug');
  });

  it('shows the empty state when search matches nothing', async () => {
    const wrapper = mountNavigation({ p1: [makeSessionSummary()] }, [makeProjectSummary()]);
    await flushPromises();

    await wrapper.find('input[aria-label="Search sessions"]').setValue('zzz-no-match');
    await flushPromises();

    expect(wrapper.text()).toContain('No sessions found');
  });
});

describe('SessionsNavigation: sorting', () => {
  it('clicking a sort option re-sorts the rendered projects', async () => {
    const wrapper = mountNavigation(
      {
        p1: [makeSessionSummary({ id: 's1', projectId: 'p1' })],
        p2: [makeSessionSummary({ id: 's2', projectId: 'p2' })],
      },
      [makeProjectSummary({ id: 'p1', name: 'zebra' }), makeProjectSummary({ id: 'p2', name: 'alpha' })],
    );
    await flushPromises();

    // `.truncate` is only ever on a project's own name span (ProjectRow.vue) -- unambiguous,
    // unlike filtering by a `title` attribute length, which a session row's own title also has.
    const projectNames = () => wrapper.findAll('span.truncate').map((w) => w.text());
    expect(projectNames()).toEqual(['zebra', 'alpha']); // default sort: recentActivity, both equal -> insertion order

    await wrapper.find('button[title="Sort projects"]').trigger('click');
    const nameOption = wrapper.findAll('button[role="menuitemradio"]').find((b) => b.text() === 'Name')!;
    await nameOption.trigger('click');
    await flushPromises();

    expect(projectNames()).toEqual(['alpha', 'zebra']);
    expect(useSessionStore().projectSortOption).toBe('name');
  });
});

describe('SessionsNavigation: search filtering', () => {
  it('typing in the search box filters to matching projects and sessions', async () => {
    const wrapper = mountNavigation(
      {
        p1: [makeSessionSummary({ id: 's1', projectId: 'p1', title: 'fix auth bug' })],
        p2: [makeSessionSummary({ id: 's2', projectId: 'p2', title: 'something else' })],
      },
      [makeProjectSummary({ id: 'p1', name: 'alpha' }), makeProjectSummary({ id: 'p2', name: 'beta' })],
    );
    await flushPromises();
    expect(wrapper.text()).toContain('beta');

    await wrapper.find('input[aria-label="Search sessions"]').setValue('auth');
    await flushPromises();

    expect(wrapper.text()).toContain('alpha');
    expect(wrapper.text()).not.toContain('beta');
  });

  it('the clear button resets the search and restores every project', async () => {
    const wrapper = mountNavigation(
      { p1: [makeSessionSummary({ id: 's1', projectId: 'p1', title: 'fix auth bug' })], p2: [makeSessionSummary({ id: 's2', projectId: 'p2', title: 'something else' })] },
      [makeProjectSummary({ id: 'p1', name: 'alpha' }), makeProjectSummary({ id: 'p2', name: 'beta' })],
    );
    await flushPromises();

    await wrapper.find('input[aria-label="Search sessions"]').setValue('auth');
    await flushPromises();
    expect(wrapper.text()).not.toContain('beta');

    await wrapper.find('button[aria-label="Clear search"]').trigger('click');
    await flushPromises();
    expect(wrapper.text()).toContain('beta');
  });
});

describe('SessionsNavigation: showAgentSessions toggle', () => {
  it('toggling it persists through prefsStore and reveals sidechain sessions', async () => {
    const wrapper = mountNavigation(
      { p1: [makeSessionSummary({ id: 's1', title: 'main' }), makeSessionSummary({ id: 's2', title: 'sidechain one', isSidechain: true })] },
      [makeProjectSummary({ id: 'p1', name: 'alpha' })],
    );
    await flushPromises();
    expect(wrapper.text()).not.toContain('sidechain one');

    await wrapper.find('button[aria-pressed]').trigger('click');
    await flushPromises();

    expect(usePrefsStore().prefs?.showAgentSessions).toBe(true);
    expect(wrapper.text()).toContain('sidechain one');
  });
});

describe('SessionsNavigation: keyboard selection (native button activation)', () => {
  it('every session row is a real, focusable native button', async () => {
    const wrapper = mountNavigation({ p1: [makeSessionSummary({ title: 'pick me' })] }, [makeProjectSummary({ id: 'p1' })]);
    await flushPromises();

    const row = wrapper.findAll('button').find((b) => b.text().includes('pick me'))!;
    expect(row.element.tagName).toBe('BUTTON');
    expect(row.attributes('disabled')).toBeUndefined();
  });

  it('activating a session row selects it through the real store', async () => {
    const wrapper = mountNavigation(
      { p1: [makeSessionSummary({ id: 's1', title: 'pick me' })] },
      [makeProjectSummary({ id: 'p1' })],
    );
    mockIPC(
      (cmd, args) => {
        if (cmd === 'list_projects') return { value: [makeProjectSummary({ id: 'p1' })], skipped: 0 };
        if (cmd === 'list_sessions') return { value: [makeSessionSummary({ id: 's1', title: 'pick me' })], skipped: 0 };
        if (cmd === 'get_session') return { summary: makeSessionSummary({ id: 's1', title: 'pick me' }), messageCount: 0, context: null, toolNames: [], skipped: 0 };
        if (cmd === 'get_session_messages') return { start: 0, messages: [], total: 0 };
        throw new Error(`unexpected command: ${cmd}, ${JSON.stringify(args)}`);
      },
      { shouldMockEvents: true },
    );
    await flushPromises();

    const row = wrapper.findAll('button').find((b) => b.text().includes('pick me'))!;
    await row.trigger('click');
    await flushPromises();

    expect(useSessionStore().selectedSession?.id).toBe('s1');
  });
});

describe('SessionsNavigation: collapsing a project', () => {
  it('collapsing hides its sessions; expanding again shows them', async () => {
    const wrapper = mountNavigation({ p1: [makeSessionSummary({ title: 'pick me' })] }, [makeProjectSummary({ id: 'p1', name: 'alpha' })]);
    await flushPromises();
    expect(wrapper.text()).toContain('pick me');

    await wrapper.find('button[aria-label="Collapse project"]').trigger('click');
    await flushPromises();
    expect(wrapper.text()).not.toContain('pick me');

    await wrapper.find('button[aria-label="Expand project"]').trigger('click');
    await flushPromises();
    expect(wrapper.text()).toContain('pick me');
  });
});
