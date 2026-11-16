// M3-51's controls on the session side: the trash on a session card (grid and list), on a project row
// and in the session toolbar. The logic is in useDeleteActions.spec.ts; this file checks each control
// sends the right command for the right item, that a declined native confirmation leaves the screen as
// it was (AC), and that the controls are named and are not buttons inside buttons.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs, SessionSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useToastsStore } from '@/stores/toasts';
import ProjectRow from './ProjectRow.vue';
import ProjectSessions from './ProjectSessions.vue';
import SessionToolbar from './SessionToolbar.vue';
import { useSessionStore, type ProjectWithSessions } from './store';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function session(id: string, title = `Session ${id}`): SessionSummary {
  return {
    id, projectId: 'p1', title, preview: null, startedAt: '2026-01-01T00:00:00Z', endedAt: null, model: null,
    tokens: { input: 0, output: 0, cacheRead: 0, cacheCreation: 0 }, turnCount: 1, isSidechain: false, parentSessionId: null,
    agentType: null, agentDescription: null, hasDebugLog: false, hasFileHistory: false, hasTodos: false,
  };
}

function project(sessions: SessionSummary[]): ProjectWithSessions {
  return { id: 'p1', name: 'alpha', path: '/p1', pathSource: 'sessionsIndex', sessionCount: sessions.length, lastActivity: null, hasMemory: false, sessions };
}

function prefs(layout: 'grid' | 'list'): Prefs {
  return {
    schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1, sessionLayout: layout,
    configProjectPath: null, configLayouts: {}, showAgentSessions: true, textEditor: 'vscode', preferredTerminal: 'terminal',
    openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true, autoExpandBlocks: true,
    parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [],
  };
}

let mounted: ReturnType<typeof mount>[] = [];
let sent: [string, unknown][];
let store: ReturnType<typeof useSessionStore>;
let toasts: ReturnType<typeof useToastsStore>;

function setUp(answer: 'written' | 'cancelled', layout: 'grid' | 'list' = 'grid') {
  setActivePinia(createPinia());
  usePrefsStore().prefs = prefs(layout);
  store = useSessionStore();
  toasts = useToastsStore();
  store.projects = [project([session('s1', 'First'), session('s2', 'Second')])];
  sent = [];
  mockIPC((cmd, args) => {
    sent.push([cmd, args]);
    if (cmd === 'delete_session' || cmd === 'delete_project') return answer;
    throw new Error(`unexpected command: ${cmd}`);
  });
}

function track<T extends ReturnType<typeof mount>>(wrapper: T): T {
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

describe.each([['grid'], ['list']] as const)('a session card (%s layout)', (layout) => {
  const mountCards = () => track(mount(ProjectSessions, { props: { project: store.projects[0] }, attachTo: document.body }));

  it('has a delete button named for its session, beside the card rather than inside it', () => {
    setUp('written', layout);
    const wrapper = mountCards();
    expect(wrapper.findAll('button[title="Delete session"]').map((b) => b.attributes('aria-label'))).toEqual(['Delete session: First', 'Delete session: Second']);
    expect(wrapper.findAll('button button')).toHaveLength(0);
  });

  it('deletes that session by its owning project and id, and says so', async () => {
    setUp('written', layout);
    const wrapper = mountCards();
    await wrapper.find('button[aria-label="Delete session: Second"]').trigger('click');
    await flushPromises();
    expect(sent).toEqual([['delete_session', { projectId: 'p1', sessionId: 's2' }]]);
    expect(store.projects[0].sessions.map((s) => s.id)).toEqual(['s1']);
    expect(toasts.current).toMatchObject({ message: 'Session deleted' });
  });

  it('leaves everything as it was when the native confirmation is declined', async () => {
    setUp('cancelled', layout);
    const wrapper = mountCards();
    await wrapper.find('button[aria-label="Delete session: Second"]').trigger('click');
    await flushPromises();
    expect(store.projects[0].sessions.map((s) => s.id)).toEqual(['s1', 's2']);
    expect(toasts.current).toBeNull();
  });

  it('does not also open the session', async () => {
    setUp('written', layout);
    const wrapper = mountCards();
    await wrapper.find('button[aria-label="Delete session: First"]').trigger('click');
    await flushPromises();
    expect(store.selectedSession).toBeNull();
  });
});

describe('a project row', () => {
  const mountRow = () => track(mount(ProjectRow, { props: { project: store.projects[0], isCollapsed: false }, attachTo: document.body }));

  it('has a delete button named for its project, beside the project button rather than inside it', () => {
    setUp('written');
    const wrapper = mountRow();
    expect(wrapper.find('button[title="Delete project"]').attributes('aria-label')).toBe('Delete project alpha');
    expect(wrapper.findAll('button button')).toHaveLength(0);
  });

  it('deletes the project by id with its sessions, and names it', async () => {
    setUp('written');
    const wrapper = mountRow();
    await wrapper.find('button[aria-label="Delete project alpha"]').trigger('click');
    await flushPromises();
    expect(sent).toEqual([['delete_project', { projectId: 'p1' }]]);
    expect(store.projects).toEqual([]);
    expect(toasts.current).toMatchObject({ message: 'Deleted alpha' });
  });

  it('leaves the project and its sessions when the native confirmation is declined', async () => {
    setUp('cancelled');
    const wrapper = mountRow();
    await wrapper.find('button[aria-label="Delete project alpha"]').trigger('click');
    await flushPromises();
    expect(store.projects.map((p) => p.id)).toEqual(['p1']);
    expect(store.projects[0].sessions).toHaveLength(2);
    expect(toasts.current).toBeNull();
  });

  it('does not also select the project', async () => {
    setUp('written');
    const wrapper = mountRow();
    await wrapper.find('button[aria-label="Delete project alpha"]').trigger('click');
    await flushPromises();
    expect(store.selectedProjectId).toBeNull();
  });
});

describe('the session toolbar', () => {
  const mountToolbar = (id: string) => track(mount(SessionToolbar, { props: { summary: store.projects[0].sessions.find((s) => s.id === id)! }, attachTo: document.body }));

  it('deletes the open session and clears the selection', async () => {
    setUp('written');
    store.selectedSession = { id: 's1', detail: { summary: store.projects[0].sessions[0], messageCount: 0, context: null, toolNames: [], skipped: 0 }, messages: [] };
    const wrapper = mountToolbar('s1');
    await wrapper.find('button[title="Delete session"]').trigger('click');
    await flushPromises();
    expect(sent).toEqual([['delete_session', { projectId: 'p1', sessionId: 's1' }]]);
    expect(store.selectedSession).toBeNull();
    expect(toasts.current).toMatchObject({ message: 'Session deleted' });
  });

  it('keeps the session open when the native confirmation is declined', async () => {
    setUp('cancelled');
    store.selectedSession = { id: 's1', detail: { summary: store.projects[0].sessions[0], messageCount: 0, context: null, toolNames: [], skipped: 0 }, messages: [] };
    const wrapper = mountToolbar('s1');
    await wrapper.find('button[title="Delete session"]').trigger('click');
    await flushPromises();
    expect(store.selectedSession?.id).toBe('s1');
    expect(store.projects[0].sessions).toHaveLength(2);
    expect(toasts.current).toBeNull();
  });
});
