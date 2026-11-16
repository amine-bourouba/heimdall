// M3-51: deleting a session or a project. The confirmation is Rust's native one, so what matters here is
// what happens to each answer: `cancelled` changes nothing and says nothing (AC), `written` and
// `unchanged` both remove the item and say which, and a failure shows its cause.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { SessionSummary } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import { useSessionStore, type ProjectWithSessions } from './store';
import { useDeleteActions } from './useDeleteActions';

function session(id: string): SessionSummary {
  return {
    id, projectId: 'p1', title: id, preview: null, startedAt: '2026-01-01T00:00:00Z', endedAt: null, model: null,
    tokens: { input: 0, output: 0, cacheRead: 0, cacheCreation: 0 }, turnCount: 1, isSidechain: false, parentSessionId: null,
    agentType: null, agentDescription: null, hasDebugLog: false, hasFileHistory: false, hasTodos: false,
  };
}

function project(id: string, sessions: string[]): ProjectWithSessions {
  return { id, name: `proj-${id}`, path: `/${id}`, pathSource: 'sessionsIndex', sessionCount: sessions.length, lastActivity: null, hasMemory: false, sessions: sessions.map(session) };
}

const busy = () => {
  throw { kind: 'busy', message: 'the file is in use by another program', path: null };
};

let sent: [string, unknown][];
let release: (value: unknown) => void = () => {};

function mockDeletes(answer: 'written' | 'cancelled' | 'unchanged' | 'error' | 'hold') {
  sent = [];
  mockIPC((cmd, args) => {
    sent.push([cmd, args]);
    if (answer === 'error') return busy();
    if (answer === 'hold') return new Promise((resolve) => (release = resolve));
    return answer;
  });
}

let store: ReturnType<typeof useSessionStore>;
let toasts: ReturnType<typeof useToastsStore>;

beforeEach(() => {
  setActivePinia(createPinia());
  store = useSessionStore();
  toasts = useToastsStore();
  store.projects = [project('p1', ['s1', 's2']), project('p2', ['s3'])];
});

describe('useDeleteActions: a session', () => {
  it('sends the owning project and the session, removes it, and says so', async () => {
    mockDeletes('written');
    await useDeleteActions().deleteSession('s1');
    expect(sent).toEqual([['delete_session', { projectId: 'p1', sessionId: 's1' }]]);
    expect(store.projects[0].sessions.map((s) => s.id)).toEqual(['s2']);
    expect(toasts.current).toMatchObject({ message: 'Session deleted', style: 'info' });
  });

  it('changes nothing and says nothing when the native confirmation is declined', async () => {
    mockDeletes('cancelled');
    await useDeleteActions().deleteSession('s1');
    expect(store.projects[0].sessions.map((s) => s.id)).toEqual(['s1', 's2']);
    expect(toasts.current).toBeNull();
  });

  it('removes a session that was already gone, and says it was', async () => {
    mockDeletes('unchanged');
    await useDeleteActions().deleteSession('s1');
    expect(store.projects[0].sessions.map((s) => s.id)).toEqual(['s2']);
    expect(toasts.current).toMatchObject({ message: 'That session was already gone', style: 'info' });
  });

  it('shows the cause, keeps the session, and can be retried, when the delete fails', async () => {
    mockDeletes('error');
    const actions = useDeleteActions();
    await actions.deleteSession('s1');
    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(store.projects[0].sessions.map((s) => s.id)).toEqual(['s1', 's2']);
    expect(actions.isBusy.value).toBe(false);
  });

  it('sends one command for a second click while the first is still open', async () => {
    mockDeletes('hold');
    const actions = useDeleteActions();
    const first = actions.deleteSession('s1');
    await actions.deleteSession('s1');
    release('written');
    await first;
    expect(sent).toHaveLength(1);
  });
});

describe('useDeleteActions: a project', () => {
  it('sends the project id, removes it with its sessions, and names it', async () => {
    mockDeletes('written');
    await useDeleteActions().deleteProject({ id: 'p1', name: 'alpha' });
    expect(sent).toEqual([['delete_project', { projectId: 'p1' }]]);
    expect(store.projects.map((p) => p.id)).toEqual(['p2']);
    expect(toasts.current).toMatchObject({ message: 'Deleted alpha', style: 'info' });
  });

  it('changes nothing and says nothing when the native confirmation is declined', async () => {
    mockDeletes('cancelled');
    await useDeleteActions().deleteProject({ id: 'p1', name: 'alpha' });
    expect(store.projects.map((p) => p.id)).toEqual(['p1', 'p2']);
    expect(toasts.current).toBeNull();
  });

  it('removes a project that was already gone, and says it was', async () => {
    mockDeletes('unchanged');
    await useDeleteActions().deleteProject({ id: 'p1', name: 'alpha' });
    expect(store.projects.map((p) => p.id)).toEqual(['p2']);
    expect(toasts.current).toMatchObject({ message: 'alpha was already gone' });
  });

  it('shows the cause and keeps the project when the delete fails', async () => {
    mockDeletes('error');
    await useDeleteActions().deleteProject({ id: 'p1', name: 'alpha' });
    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(store.projects.map((p) => p.id)).toEqual(['p1', 'p2']);
  });
});
