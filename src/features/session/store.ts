// Ported from AppState.swift:82-120,240-330 (projects/sessions, selection, the session-loading
// race-guard state machine, tool filter and session search state) and ProjectSortOption/
// SessionLayout (ProjectSortingTests.swift). `sessionLayout`/`showAgentSessions` are NOT
// duplicated here -- both already live in the one Rust-owned Prefs struct (ADR 0007, M2-05's
// prefsStore), read from there directly. `fontScale`/toasts are M2-05's own stores; navigation
// history is the router's native history (M2-06's useNavigation, port-plan P-3) -- none of
// AppState's own hand-rolled versions of those three concerns are re-ported here.
//
// Heimdall's data shape is not Poirot's: `Project.sessions` there is `[Session]` with messages
// already attached; here a project eagerly loads only `SessionSummary[]` (list-row data, no
// message content -- confirmed against docs/spec/session-views.md §3/§4's own row descriptions,
// which only ever need title/preview/token/turn-count fields). A session's full content
// (`SessionDetail` + paged `Message[]`) is fetched only once it is actually selected.

import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc, onDataChanged } from '@/ipc/client';
import type { DataChange, Message, ProjectSummary, SessionDetail, SessionSummary } from '@/ipc/bindings';
import { bestScore, fuzzyMatch } from '@/lib/fuzzyMatch';
import { usePrefsStore } from '@/stores/prefs';

export type ProjectSortOption = 'recentActivity' | 'name' | 'sessionCount';

export interface ProjectWithSessions extends ProjectSummary {
  sessions: SessionSummary[];
}

/** A session once opened: its header (`SessionDetail`) plus however many message pages have
 * loaded so far. `AppStateSessionLoadingTests`' own `Session` stand-in, adapted to this split. */
export interface LoadedSession {
  id: string;
  detail: SessionDetail;
  messages: Message[];
}

/** `SessionDetailView.swift:7`'s own `pageSize` (docs/spec/session-views.md §1). */
const MESSAGE_PAGE_SIZE = 20;

function mostRecentStartedAt(sessions: SessionSummary[]): string {
  return sessions.reduce((latest, s) => (s.startedAt > latest ? s.startedAt : latest), '');
}

export const useSessionStore = defineStore('session', () => {
  const prefsStore = usePrefsStore();

  const projects = ref<ProjectWithSessions[]>([]);
  const isLoadingProjects = ref(true);
  const isLoadingMoreProjects = ref(true);
  let subscribed = false;

  const selectedProjectId = ref<string | null>(null);
  const selectedSession = ref<LoadedSession | null>(null);
  const isLoadingSession = ref(false);
  const sessionCache = new Map<string, LoadedSession>();

  const projectSortOption = ref<ProjectSortOption>('recentActivity');
  const sidebarSearchQuery = ref('');

  const sessionSearchQuery = ref('');
  const isSessionSearchActive = ref(false);
  const isToolFilterActive = ref(false);
  const activeToolFilters = ref<Set<string>>(new Set());
  // AppState.swift:301,198,219 -- deliberately left unported by M2-14 ("session-detail-only UI
  // state... for a later task to add"); M2-22 (session toolbar) is the first to need all three.
  const allBlocksExpanded = ref(false);
  const isShowingFileHistory = ref(false);
  const showDebugLogSessionId = ref<string | null>(null);
  const isShowingExportOptions = ref(false);

  const currentProject = computed(() => projects.value.find((p) => p.id === selectedProjectId.value) ?? null);

  // Ported from AppState.swift:310-360 (`filteredSortedProjects`).
  const filteredSortedProjects = computed<ProjectWithSessions[]>(() => {
    const showAgentSessions = prefsStore.prefs?.showAgentSessions ?? false;
    const visible = showAgentSessions
      ? projects.value
      : projects.value.map((p) => ({ ...p, sessions: p.sessions.filter((s) => !s.isSidechain) }));
    const nonEmpty = visible.filter((p) => p.sessions.length > 0);

    const trimmed = sidebarSearchQuery.value.trim();
    const searched =
      trimmed === ''
        ? nonEmpty
        : nonEmpty.flatMap((p) => {
            if (fuzzyMatch(p.name, trimmed)) return [p];
            const matchingSessions = p.sessions.filter((s) => fuzzyMatch(s.title, trimmed) ?? fuzzyMatch(s.id, trimmed));
            return matchingSessions.length > 0 ? [{ ...p, sessions: matchingSessions }] : [];
          });

    if (trimmed !== '') {
      return [...searched].sort(
        (a, b) =>
          bestScore(
            b.name,
            b.sessions.map((s) => s.title),
            trimmed,
          ) -
          bestScore(
            a.name,
            a.sessions.map((s) => s.title),
            trimmed,
          ),
      );
    }

    switch (projectSortOption.value) {
      case 'recentActivity':
        return [...searched].sort((a, b) => mostRecentStartedAt(b.sessions).localeCompare(mostRecentStartedAt(a.sessions)));
      case 'name':
        return [...searched].sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
      case 'sessionCount':
        return [...searched].sort((a, b) => b.sessions.length - a.sessions.length);
      default:
        return searched;
    }
  });

  async function load() {
    isLoadingProjects.value = true;
    isLoadingMoreProjects.value = true;
    const summaries = (await ipc.listProjects()).value;
    projects.value = await Promise.all(
      summaries.map(async (summary) => ({ ...summary, sessions: (await ipc.listSessions(summary.id, true)).value })),
    );
    isLoadingProjects.value = false;
    isLoadingMoreProjects.value = false;

    if (!subscribed) {
      subscribed = true;
      await onDataChanged(['projects', 'sessions'], (changes) => void handleDataChanged(changes));
    }
  }

  async function handleDataChanged(changes: DataChange[]) {
    for (const change of changes) {
      if (change.area === 'projects' || change.area === 'all') {
        await load();
        return;
      }
      if (change.area === 'sessions' && change.key) {
        // AC3: a project not currently loaded (not "on screen") gets no reload at all.
        if (!projects.value.some((p) => p.id === change.key)) continue;
        const sessions = (await ipc.listSessions(change.key, true)).value;
        projects.value = projects.value.map((p) => (p.id === change.key ? { ...p, sessions } : p));
      }
    }
  }

  function cacheSession(session: LoadedSession) {
    sessionCache.set(session.id, session);
  }

  function cachedSession(id: string): LoadedSession | undefined {
    return sessionCache.get(id);
  }

  function deselectSession() {
    selectedSession.value = null;
    isLoadingSession.value = false;
    isShowingFileHistory.value = false;
  }

  // Ported from the race this session's own AppStateSessionLoadingTests guard against: a stale
  // load resolving after a *newer* selection must not stomp on it or reset its loading flag.
  // Poirot's own guard reads `selectedSession?.id == sessionId` in a `defer`; same check here.
  async function selectSession(projectId: string, sessionId: string) {
    // AppState.swift:94 -- selecting a (possibly different) session always resets this.
    isShowingFileHistory.value = false;
    selectedProjectId.value = projectId;
    const cached = cachedSession(sessionId);
    if (cached) {
      selectedSession.value = cached;
      isLoadingSession.value = false;
      return;
    }

    const summary = projects.value.find((p) => p.id === projectId)?.sessions.find((s) => s.id === sessionId);
    selectedSession.value = summary
      ? { id: sessionId, detail: { summary, messageCount: 0, context: null, toolNames: [], skipped: 0 }, messages: [] }
      : null;
    isLoadingSession.value = true;

    try {
      const [detail, page] = await Promise.all([
        ipc.getSession(projectId, sessionId),
        ipc.getSessionMessages(projectId, sessionId, 0, MESSAGE_PAGE_SIZE),
      ]);
      const full: LoadedSession = { id: sessionId, detail, messages: page.messages };
      cacheSession(full);
      selectedSession.value = full;
    } finally {
      if (selectedSession.value?.id === sessionId) isLoadingSession.value = false;
    }
  }

  /** `SessionDetailView.swift:300`'s "load more": another page of 20, appended. */
  async function loadMoreMessages() {
    const current = selectedSession.value;
    if (!current || !selectedProjectId.value) return;
    if (current.messages.length >= current.detail.messageCount) return;
    const page = await ipc.getSessionMessages(selectedProjectId.value, current.id, current.messages.length, MESSAGE_PAGE_SIZE);
    if (selectedSession.value?.id !== current.id) return; // navigated away meanwhile
    const updated: LoadedSession = { ...current, messages: [...current.messages, ...page.messages] };
    selectedSession.value = updated;
    cacheSession(updated);
  }

  // Ported from AppState.swift:443-464. Poirot finds the owning project by searching, since the
  // view only ever has the `Session` itself on hand -- same shape here, just an id.
  async function deleteSession(sessionId: string) {
    const owner = projects.value.find((p) => p.sessions.some((s) => s.id === sessionId));
    if (owner) await ipc.deleteSession(owner.id, sessionId);

    if (selectedSession.value?.id === sessionId) selectedSession.value = null;
    sessionCache.delete(sessionId);
    projects.value = projects.value.map((p) => ({ ...p, sessions: p.sessions.filter((s) => s.id !== sessionId) })).filter((p) => p.sessions.length > 0);
  }

  // Ported from AppState.swift:379-396.
  async function deleteProject(projectId: string) {
    await ipc.deleteProject(projectId);

    const project = projects.value.find((p) => p.id === projectId);
    if (project) {
      if (selectedSession.value && project.sessions.some((s) => s.id === selectedSession.value!.id)) {
        selectedSession.value = null;
      }
      for (const s of project.sessions) sessionCache.delete(s.id);
    }
    projects.value = projects.value.filter((p) => p.id !== projectId);
  }

  return {
    projects,
    isLoadingProjects,
    isLoadingMoreProjects,
    selectedProjectId,
    selectedSession,
    isLoadingSession,
    sessionCache,
    projectSortOption,
    sidebarSearchQuery,
    sessionSearchQuery,
    isSessionSearchActive,
    isToolFilterActive,
    activeToolFilters,
    allBlocksExpanded,
    isShowingFileHistory,
    showDebugLogSessionId,
    isShowingExportOptions,
    currentProject,
    filteredSortedProjects,
    load,
    handleDataChanged,
    cacheSession,
    cachedSession,
    deselectSession,
    selectSession,
    loadMoreMessages,
    deleteSession,
    deleteProject,
  };
});
