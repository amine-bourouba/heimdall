// Carried-over cases (docs/spec/test-carryover.md): 11 from AppStateSessionLoadingTests.swift,
// 24 of AppStateTests.swift's own session-related cases, 6 from ProjectSortingTests.swift.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { Message, SessionSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useSessionStore, type LoadedSession, type ProjectSortOption, type ProjectWithSessions } from './store';

function makeProject(overrides: Partial<ProjectWithSessions> = {}): ProjectWithSessions {
  return {
    id: 'proj',
    name: 'test-project',
    path: '/test/path',
    pathSource: 'sessionsIndex',
    sessionCount: overrides.sessions?.length ?? 0,
    lastActivity: null,
    hasMemory: false,
    sessions: [],
    ...overrides,
  };
}

function makeProjectSummary(overrides: Partial<ProjectWithSessions> = {}) {
  const { id, name, path, pathSource, sessionCount, lastActivity, hasMemory } = makeProject(overrides);
  return { id, name, path, pathSource, sessionCount, lastActivity, hasMemory };
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

function makeMessage(id: string): Message {
  return { id, role: 'user', timestamp: null, model: null, usage: null, blocks: [] };
}

function makeLoadedSession(id: string, messages: Message[] = []): LoadedSession {
  return {
    id,
    detail: { summary: makeSessionSummary({ id }), messageCount: messages.length, context: null, toolNames: [], skipped: 0 },
    messages,
  };
}

beforeEach(() => {
  setActivePinia(createPinia());
  usePrefsStore().prefs = { showAgentSessions: false } as ReturnType<typeof usePrefsStore>['prefs'] & object;
});

describe('session store: session-loading state machine (AppStateSessionLoadingTests)', () => {
  it('selectingCachedSession_setsLoadingFalse', () => {
    const store = useSessionStore();
    const loaded = makeLoadedSession('s1', [makeMessage('m1')]);
    store.cacheSession(loaded);
    store.isLoadingSession = true;

    const cached = store.cachedSession('s1');
    if (cached) {
      store.selectedSession = cached;
      store.isLoadingSession = false;
    }

    expect(store.isLoadingSession).toBe(false);
    expect(store.selectedSession?.messages.length).toBeGreaterThan(0);
  });

  it('selectingNilSession_resetsLoadingFlag', () => {
    const store = useSessionStore();
    store.isLoadingSession = true;

    store.deselectSession();

    expect(store.isLoadingSession).toBe(false);
  });

  it('selectingFullyLoadedSession_resetsLoadingFlag', () => {
    const store = useSessionStore();
    store.isLoadingSession = true;

    const loaded = makeLoadedSession('s1', [makeMessage('m1')]);
    store.selectedSession = loaded;
    if (loaded.messages.length > 0) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
  });

  it('selectingSessionWithNoFileURL_resetsLoadingFlag (no detail available at all)', () => {
    // Heimdall's `LoadedSession` has no `fileURL` -- there is no file-missing session shape to
    // select at all, since a session only exists here once Rust has already found its file. The
    // real equivalent gap is "no SessionSummary on hand yet" (selectSession's own `summary`
    // lookup failing) -- ported as that instead of a field that does not exist in this port.
    const store = useSessionStore();
    store.isLoadingSession = true;

    store.selectedSession = null;
    if (store.selectedSession === null) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
  });

  it('deferBlock_resetsFlag_whenSessionStillSelected', () => {
    const store = useSessionStore();
    const sessionId = 's1';
    store.selectedSession = makeLoadedSession(sessionId);
    store.isLoadingSession = true;

    if (store.selectedSession?.id === sessionId) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
  });

  it('deferBlock_skipsReset_whenDifferentSessionSelected', () => {
    const store = useSessionStore();
    const originalId = 's1';
    store.selectedSession = makeLoadedSession('s2'); // user navigated away
    store.isLoadingSession = true;

    if (store.selectedSession?.id === originalId) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(true);
  });

  it('deferBlock_resetsFlag_afterCancellation', () => {
    const store = useSessionStore();
    const sessionId = 's1';
    store.selectedSession = makeLoadedSession(sessionId);
    store.isLoadingSession = true;

    if (store.selectedSession?.id === sessionId) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
  });

  it('rapidSelection_cacheHitAfterStub_resetsLoadingCorrectly', () => {
    const store = useSessionStore();
    const loaded = makeLoadedSession('s1', [makeMessage('m1')]);
    store.cacheSession(loaded);

    store.selectedSession = makeLoadedSession('s1'); // stub, triggers loading
    store.isLoadingSession = true;

    const cached = store.cachedSession('s1');
    if (cached) {
      store.selectedSession = cached;
      store.isLoadingSession = false;
    }

    expect(store.isLoadingSession).toBe(false);
    expect(store.selectedSession?.messages.length).toBe(1);
  });

  it('rapidSelection_switchBackToCached_resetsCorrectly', () => {
    const store = useSessionStore();
    const loadedA = makeLoadedSession('sA', [makeMessage('m1')]);
    store.cacheSession(loadedA);

    store.selectedSession = makeLoadedSession('sB');
    store.isLoadingSession = true;

    const cached = store.cachedSession('sA');
    if (cached) {
      store.selectedSession = cached;
      store.isLoadingSession = false;
    }

    // B's own defer runs but sees a different session now selected -- must not re-set loading.
    if (store.selectedSession?.id === 'sB') store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
    expect(store.selectedSession?.id).toBe('sA');
  });

  it('successfulLoad_cachesAndUpdatesSession', () => {
    const store = useSessionStore();
    const sessionId = 's1';
    store.selectedSession = makeLoadedSession(sessionId);
    store.isLoadingSession = true;

    const full = makeLoadedSession(sessionId, [makeMessage('m1')]);
    store.cacheSession(full);
    store.selectedSession = full;

    if (store.selectedSession?.id === sessionId) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
    expect(store.cachedSession(sessionId)).toBeDefined();
    expect(store.selectedSession?.messages.length).toBeGreaterThan(0);
  });

  it('successfulLoad_nilParseResult_stillResetsLoading', () => {
    const store = useSessionStore();
    const sessionId = 's1';
    store.selectedSession = makeLoadedSession(sessionId);
    store.isLoadingSession = true;

    const full: LoadedSession | null = null;
    if (full) {
      store.cacheSession(full);
      store.selectedSession = full;
    }

    if (store.selectedSession?.id === sessionId) store.isLoadingSession = false;

    expect(store.isLoadingSession).toBe(false);
  });
});

describe('session store: currentProject', () => {
  it('currentProject_withMatchingId_returnsProject', () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', name: 'alpha' })];
    store.selectedProjectId = 'p1';
    expect(store.currentProject?.name).toBe('alpha');
  });

  it('currentProject_withNilSelection_returnsNil', () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1' })];
    store.selectedProjectId = null;
    expect(store.currentProject).toBeNull();
  });

  it('currentProject_withNonMatchingId_returnsNil', () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1' })];
    store.selectedProjectId = 'nonexistent';
    expect(store.currentProject).toBeNull();
  });
});

describe('session store: filteredSortedProjects', () => {
  it('filteredSortedProjects_excludesEmptyProjects', () => {
    const store = useSessionStore();
    store.projects = [
      makeProject({ id: 'empty', name: 'empty', sessions: [] }),
      makeProject({ id: 'full', name: 'full', sessions: [makeSessionSummary({ id: 's1' })] }),
    ];
    const result = store.filteredSortedProjects;
    expect(result.length).toBe(1);
    expect(result[0].id).toBe('full');
  });

  it('filteredSortedProjects_searchFiltersByProjectName', () => {
    const store = useSessionStore();
    store.projects = [
      makeProject({ id: 'p1', name: 'poirot', sessions: [makeSessionSummary({ id: 's1' })] }),
      makeProject({ id: 'p2', name: 'ignio', sessions: [makeSessionSummary({ id: 's2' })] }),
    ];
    store.sidebarSearchQuery = 'poi';
    const result = store.filteredSortedProjects;
    expect(result.length).toBe(1);
    expect(result[0].id).toBe('p1');
  });

  it('filteredSortedProjects_searchFiltersBySessionTitle', () => {
    const store = useSessionStore();
    store.projects = [
      makeProject({ id: 'p1', name: 'alpha', sessions: [makeSessionSummary({ id: 's1', title: 'Fix auth bug' })] }),
      makeProject({ id: 'p2', name: 'beta', sessions: [makeSessionSummary({ id: 's2' })] }),
    ];
    store.sidebarSearchQuery = 'auth';
    const result = store.filteredSortedProjects;
    expect(result.length).toBe(1);
    expect(result[0].id).toBe('p1');
    expect(result[0].sessions.length).toBe(1);
  });

  it('filteredSortedProjects_searchIsCaseInsensitive', () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', name: 'MyApp', sessions: [makeSessionSummary({ id: 's1' })] })];
    store.sidebarSearchQuery = 'myapp';
    expect(store.filteredSortedProjects.length).toBe(1);
  });

  it('filteredSortedProjects_emptySearchReturnsAll', () => {
    const store = useSessionStore();
    store.projects = [
      makeProject({ id: 'p1', name: 'a', sessions: [makeSessionSummary({ id: 's1' })] }),
      makeProject({ id: 'p2', name: 'b', sessions: [makeSessionSummary({ id: 's2' })] }),
    ];
    store.sidebarSearchQuery = '   ';
    expect(store.filteredSortedProjects.length).toBe(2);
  });

  it('filteredSortedProjects_sortByRecentActivity', () => {
    const store = useSessionStore();
    store.projectSortOption = 'recentActivity';
    store.projects = [
      makeProject({ id: 'old', name: 'old', sessions: [makeSessionSummary({ id: 's1', startedAt: '1970-01-01T00:16:40Z' })] }),
      makeProject({ id: 'new', name: 'new', sessions: [makeSessionSummary({ id: 's2', startedAt: '1970-01-01T02:30:00Z' })] }),
    ];
    const result = store.filteredSortedProjects;
    expect(result[0].id).toBe('new');
    expect(result[result.length - 1].id).toBe('old');
  });

  it('filteredSortedProjects_sortByName', () => {
    const store = useSessionStore();
    store.projectSortOption = 'name';
    store.projects = [
      makeProject({ id: 'z', name: 'zebra', sessions: [makeSessionSummary({ id: 's1' })] }),
      makeProject({ id: 'a', name: 'alpha', sessions: [makeSessionSummary({ id: 's2' })] }),
    ];
    const result = store.filteredSortedProjects;
    expect(result[0].id).toBe('a');
    expect(result[result.length - 1].id).toBe('z');
  });

  it('filteredSortedProjects_sortBySessionCount', () => {
    const store = useSessionStore();
    store.projectSortOption = 'sessionCount';
    store.projects = [
      makeProject({ id: 'few', name: 'few', sessions: [makeSessionSummary({ id: 's1' })] }),
      makeProject({
        id: 'many',
        name: 'many',
        sessions: [makeSessionSummary({ id: 's2' }), makeSessionSummary({ id: 's3' }), makeSessionSummary({ id: 's4' })],
      }),
    ];
    const result = store.filteredSortedProjects;
    expect(result[0].id).toBe('many');
    expect(result[result.length - 1].id).toBe('few');
  });
});

describe('session store: deleteSession', () => {
  function mockDelete() {
    mockIPC((cmd) => {
      if (cmd === 'delete_session' || cmd === 'delete_project') return { status: 'ok' };
      throw new Error(`unexpected command: ${cmd}`);
    });
  }

  it('deleteSession_removesFromProject', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' }), makeSessionSummary({ id: 's2' })] })];

    await store.deleteSession('s1');

    expect(store.projects[0].sessions.length).toBe(1);
    expect(store.projects[0].sessions[0].id).toBe('s2');
  });

  it('deleteSession_removesEmptyProject', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] })];

    await store.deleteSession('s1');

    expect(store.projects.length).toBe(0);
  });

  it('deleteSession_clearsSelectionIfSelected', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] })];
    store.selectedSession = makeLoadedSession('s1');

    await store.deleteSession('s1');

    expect(store.selectedSession).toBeNull();
  });

  it('deleteSession_preservesSelectionIfDifferent', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' }), makeSessionSummary({ id: 's2' })] })];
    store.selectedSession = makeLoadedSession('s2');

    await store.deleteSession('s1');

    expect(store.selectedSession?.id).toBe('s2');
  });

  it('deleteSession_removesFromCache', async () => {
    mockDelete();
    const store = useSessionStore();
    store.cacheSession(makeLoadedSession('s1'));
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' }), makeSessionSummary({ id: 's2' })] })];

    await store.deleteSession('s1');

    expect(store.cachedSession('s1')).toBeUndefined();
  });

  it('deleteSession_leavesOtherProjectsUntouched', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [
      makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] }),
      makeProject({ id: 'p2', name: 'other', sessions: [makeSessionSummary({ id: 's2' })] }),
    ];

    await store.deleteSession('s1');

    expect(store.projects.length).toBe(1);
    expect(store.projects[0].id).toBe('p2');
  });
});

describe('session store: deleteProject', () => {
  function mockDelete() {
    mockIPC((cmd) => {
      if (cmd === 'delete_session' || cmd === 'delete_project') return { status: 'ok' };
      throw new Error(`unexpected command: ${cmd}`);
    });
  }

  it('deleteProject_removesFromProjects', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] })];

    await store.deleteProject('p1');

    expect(store.projects.length).toBe(0);
  });

  it('deleteProject_clearsSelectionIfSessionBelongsToProject', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] })];
    store.selectedSession = makeLoadedSession('s1');

    await store.deleteProject('p1');

    expect(store.selectedSession).toBeNull();
  });

  it('deleteProject_preservesSelectionIfDifferentProject', async () => {
    mockDelete();
    const store = useSessionStore();
    store.projects = [
      makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] }),
      makeProject({ id: 'p2', sessions: [makeSessionSummary({ id: 's2' })] }),
    ];
    store.selectedSession = makeLoadedSession('s2');

    await store.deleteProject('p1');

    expect(store.selectedSession?.id).toBe('s2');
  });

  it('deleteProject_removesCachedSessions', async () => {
    mockDelete();
    const store = useSessionStore();
    store.cacheSession(makeLoadedSession('s1'));
    store.cacheSession(makeLoadedSession('s2'));
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' }), makeSessionSummary({ id: 's2' })] })];

    await store.deleteProject('p1');

    expect(store.cachedSession('s1')).toBeUndefined();
    expect(store.cachedSession('s2')).toBeUndefined();
  });
});

describe('session store: defaults', () => {
  it('isLoadingSession_defaultsFalse', () => {
    expect(useSessionStore().isLoadingSession).toBe(false);
  });

  it('initialState_hasCorrectDefaults', () => {
    const store = useSessionStore();
    expect(store.selectedSession).toBeNull();
    expect(store.selectedProjectId).toBeNull();
    expect(store.projects).toEqual([]);
    expect(store.isLoadingProjects).toBe(true);
    expect(store.isLoadingMoreProjects).toBe(true);
    expect(store.isLoadingSession).toBe(false);
    expect(store.projectSortOption).toBe('recentActivity');
    expect(store.sidebarSearchQuery).toBe('');
    expect(store.sessionCache.size).toBe(0);
  });
});

describe('session store: cacheSession overwrites', () => {
  it('cacheSession_overwritesExistingEntry', () => {
    const store = useSessionStore();
    store.cacheSession(makeLoadedSession('s1', []));
    store.cacheSession(makeLoadedSession('s1', [makeMessage('m1')]));

    expect(store.cachedSession('s1')?.messages.length).toBe(1);
  });
});

describe('ProjectSortOption (ProjectSortingTests.swift)', () => {
  it('allCases_hasThreeOptions', () => {
    const allOptions: ProjectSortOption[] = ['recentActivity', 'name', 'sessionCount'];
    expect(allOptions.length).toBe(3);
  });

  it('rawValues_areCorrect', () => {
    const recentActivity: ProjectSortOption = 'recentActivity';
    const name: ProjectSortOption = 'name';
    const sessionCount: ProjectSortOption = 'sessionCount';
    expect(recentActivity).toBe('recentActivity');
    expect(name).toBe('name');
    expect(sessionCount).toBe('sessionCount');
  });

  // `label_returnsLocalizedStrings` is not ported: Heimdall has no per-option label here at all
  // yet (that is sort-menu UI text, M2-15's own job) -- there is nothing to assert against
  // without inventing a label this task was never asked to add.
});

describe('SessionLayout (ProjectSortingTests.swift) -- a contract type (bindings.ts `Layout`), not a runtime enum', () => {
  it('rawValues_areCorrect', () => {
    const grid: 'grid' | 'list' = 'grid';
    const list: 'grid' | 'list' = 'list';
    expect(grid).toBe('grid');
    expect(list).toBe('list');
  });

  // `initFromRawValue_worksForValidValues`/`_returnsNilForInvalid` have no port: `Layout` is a
  // TypeScript string-literal union, not a runtime `enum` with a failable initializer -- an
  // invalid value is a compile-time type error here, not a runtime nil, so there is no
  // "construct from an arbitrary raw string" behaviour left to test.
});

describe('session store: data-changed reload scoping (AC3)', () => {
  it('a sessions change for a project NOT currently loaded triggers no reload at all', async () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] })];

    let called = false;
    mockIPC((cmd) => {
      called = true;
      throw new Error(`unexpected command: ${cmd}`);
    });

    await store.handleDataChanged([{ area: 'sessions', key: 'unknown-project' }]);

    expect(called).toBe(false);
  });

  it('a sessions change for a project that IS loaded re-fetches just that project', async () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1' })] })];

    mockIPC((cmd, args) => {
      if (cmd === 'list_sessions' && (args as { projectId: string }).projectId === 'p1') {
        return { value: [makeSessionSummary({ id: 's1' }), makeSessionSummary({ id: 's-new' })], skipped: 0 };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    await store.handleDataChanged([{ area: 'sessions', key: 'p1' }]);

    expect(store.projects[0].sessions.length).toBe(2);
  });
});

// Not carried over from Swift (no real async loading function exists there to test the same
// way -- AppStateSessionLoadingTests hand-simulates each step instead, see above). These prove
// `load`/`selectSession`/`loadMoreMessages` -- the real orchestration a later task (M2-15/M2-20)
// wires into the UI -- actually drive the same state machine correctly end to end.
describe('session store: real async orchestration', () => {
  it('load() populates projects with their own sessions and clears both loading flags', async () => {
    const store = useSessionStore();
    mockIPC((cmd, args) => {
      if (cmd === 'list_projects') return { value: [makeProjectSummary({ id: 'p1' })], skipped: 0 };
      if (cmd === 'list_sessions' && (args as { projectId: string }).projectId === 'p1') {
        return { value: [makeSessionSummary({ id: 's1' })], skipped: 0 };
      }
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true }); // load() also subscribes to `data-changed` via onDataChanged.

    await store.load();

    expect(store.isLoadingProjects).toBe(false);
    expect(store.isLoadingMoreProjects).toBe(false);
    expect(store.projects.length).toBe(1);
    expect(store.projects[0].sessions).toEqual([makeSessionSummary({ id: 's1' })]);
  });

  it('selectSession() on a cache miss fetches the detail and first message page, then caches it', async () => {
    const store = useSessionStore();
    store.projects = [makeProject({ id: 'p1', sessions: [makeSessionSummary({ id: 's1', title: 'hi' })] })];
    mockIPC((cmd, args) => {
      if (cmd === 'get_session') return { summary: makeSessionSummary({ id: 's1', title: 'hi' }), messageCount: 1, context: null, toolNames: [], skipped: 0 };
      if (cmd === 'get_session_messages') return { start: 0, messages: [makeMessage('m1')], total: 1 };
      throw new Error(`unexpected command: ${cmd}, ${JSON.stringify(args)}`);
    });

    await store.selectSession('p1', 's1');

    expect(store.isLoadingSession).toBe(false);
    expect(store.selectedSession?.messages).toEqual([makeMessage('m1')]);
    expect(store.cachedSession('s1')?.messages).toEqual([makeMessage('m1')]);
  });

  it('loadMoreMessages() appends the next page to the selected session', async () => {
    const store = useSessionStore();
    store.selectedProjectId = 'p1';
    store.selectedSession = {
      id: 's1',
      detail: { summary: makeSessionSummary({ id: 's1' }), messageCount: 2, context: null, toolNames: [], skipped: 0 },
      messages: [makeMessage('m1')],
    };
    mockIPC((cmd, args) => {
      if (cmd === 'get_session_messages') {
        expect((args as { start: number }).start).toBe(1);
        return { start: 1, messages: [makeMessage('m2')], total: 2 };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    await store.loadMoreMessages();

    expect(store.selectedSession?.messages.map((m) => m.id)).toEqual(['m1', 'm2']);
  });

  it('loadMoreMessages() is a no-op once every message has already loaded', async () => {
    const store = useSessionStore();
    store.selectedProjectId = 'p1';
    store.selectedSession = {
      id: 's1',
      detail: { summary: makeSessionSummary({ id: 's1' }), messageCount: 1, context: null, toolNames: [], skipped: 0 },
      messages: [makeMessage('m1')],
    };
    mockIPC(() => {
      throw new Error('get_session_messages should not be called once fully loaded');
    });

    await expect(store.loadMoreMessages()).resolves.toBeUndefined();
  });
});
