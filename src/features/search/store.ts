// Ported from the state half of SearchOverlayView.swift (docs/spec/search-overlay.md, M3-01).
// The 16-category server search itself lives entirely in Rust (M1-29/M1-30's `search` command) --
// this store owns overlay open/close, the typing debounce and staleness guard (§8), the empty-
// query state's two lists (§3), and keyboard selection (§6). It does not call `useRouter()` or
// `useSessionStore()` itself: every other store in this codebase keeps navigation in the
// component that actually has router context (session/store.ts's own header comment -- "the
// store stays the single place selection actually happens... this component only mirrors it"),
// so `activate()`/`activateEmptyRow()` return a plain descriptor and SearchOverlay.vue performs
// the real `router.push`/session-store call.

import { computed, ref, watch } from 'vue';
import { defineStore } from 'pinia';
import type { RouteLocationRaw } from 'vue-router';
import { ipc } from '@/ipc/client';
import type { ProjectContext, RecentSession, SearchAction, SearchGroup, SearchHit, SearchMode, SidebarCounts } from '@/ipc/bindings';
import { navigationItems } from '@/router/navigationItems';
import { usePrefsStore } from '@/stores/prefs';
import * as trayFeature from '@/features/tray';
import * as mcpFeature from '@/features/config/mcp';

/** §8's own 200ms typing debounce. */
const TYPING_DEBOUNCE_MS = 200;
/** §3: "the 5 most recent sessions across all projects". */
const RECENT_SESSION_LIMIT = 5;

/** What activating a row should do, resolved by the store but performed by the component that
 * owns router/session-store access (see header comment). */
export type SearchActivation =
  | { kind: 'route'; to: RouteLocationRaw }
  | { kind: 'session'; projectId: string; sessionId: string; debugLog?: boolean }
  | { kind: 'none' };

/** §4: both quick actions are deferred at parity (docs/parity-matrix.md rows S17/S18, "HD until
 * the tray/wizard ships") -- gated on each owning feature's own `available` flag, which is false
 * for both today (M3-58, M3-48 are still pending), so this is exercised by its own test but
 * otherwise inert until those features land. Deliberately *not* folded into `flatResults`/
 * keyboard selection: doing so now would be complexity with nothing to verify it against, since
 * neither flag can be true yet -- click-to-activate is enough until one of them flips. */
export interface QuickAction {
  label: string;
  subtitle: string;
  icon: string;
  group: 'commands' | 'mcpServers';
}

const QUICK_ACTION_DEFS: QuickAction[] = [
  { label: 'Toggle Menu Bar Icon', subtitle: 'Show or hide the menu bar icon', icon: 'menubar.rectangle', group: 'commands' },
  { label: 'Add MCP Server', subtitle: 'Open the MCP server setup wizard', icon: 'plus.circle', group: 'mcpServers' },
];

export const useSearchStore = defineStore('search', () => {
  const prefsStore = usePrefsStore();

  const isOpen = ref(false);
  const query = ref('');
  const debouncedQuery = ref('');
  const isSearching = ref(false);
  const selectedIndex = ref(0);
  const groups = ref<SearchGroup[]>([]);
  const recentSessions = ref<RecentSession[]>([]);
  const sidebarCounts = ref<SidebarCounts | null>(null);

  let debounceTimer: ReturnType<typeof setTimeout> | undefined;
  // A monotonic generation counter instead of AbortController: mirrors the Swift reference app's own
  // Task-cancellation guard ("checks Task.isCancelled before committing", §8) with no extra type.
  let searchGeneration = 0;

  const mode = computed<SearchMode>(() => prefsStore.prefs?.globalSearchMode ?? 'fuzzy');

  // §3.1: `configNavItems` = NavigationItem.allItems with id != "sessions" filtered out (13 items).
  const emptyScreens = computed(() => navigationItems.filter((item) => item.id !== 'sessions'));

  const flatResults = computed<SearchHit[]>(() => groups.value.flatMap((g) => g.hits));

  const isEmptyQuery = computed(() => query.value.trim() === '');

  // §4: available only once the query matches the label text, same as any other scored row --
  // a plain case-insensitive substring is enough fidelity for a row that cannot render yet.
  const quickActions = computed<QuickAction[]>(() => {
    if (isEmptyQuery.value) return [];
    const trimmed = query.value.trim().toLowerCase();
    const available: Record<QuickAction['group'], boolean> = { commands: trayFeature.available, mcpServers: mcpFeature.available };
    return QUICK_ACTION_DEFS.filter((def) => available[def.group] && def.label.toLowerCase().includes(trimmed));
  });

  /** The side effect itself only runs here, on an explicit click/Enter -- never inside the
   * `quickActions` computed above, which must stay pure (it re-evaluates on every keystroke that
   * still matches, not just once). */
  function activateQuickAction(action: QuickAction): SearchActivation {
    if (action.group === 'commands') {
      void prefsStore.update({ showMenuBarIcon: !(prefsStore.prefs?.showMenuBarIcon ?? true) });
      return { kind: 'none' };
    }
    return navigateToScreenActivation('mcpServers');
  }

  function projectContext(): ProjectContext | null {
    // Same convention as Sidebar.vue's own `projectContext()` (M2-06): no session-selection-
    // derived "current project" concept exists yet, only a picked folder.
    return prefsStore.prefs?.configProjectPath ? { kind: 'pickedFolder' } : null;
  }

  /** §1 preamble: config-derived categories load once per overlay open, not per keystroke. The
   * 16-category search itself has no such staleness (§8's own note: Heimdall's `search` re-reads
   * on every call, "a strict improvement, not a behaviour to port"), so only these two need it. */
  async function open() {
    isOpen.value = true;
    query.value = '';
    debouncedQuery.value = '';
    groups.value = [];
    selectedIndex.value = 0;
    const [counts, recent] = await Promise.all([ipc.getSidebarCounts(projectContext()), ipc.listRecentSessions(RECENT_SESSION_LIMIT)]);
    sidebarCounts.value = counts;
    recentSessions.value = recent;
  }

  function close() {
    isOpen.value = false;
    if (debounceTimer !== undefined) clearTimeout(debounceTimer);
  }

  function toggleOpen() {
    if (isOpen.value) close();
    else void open();
  }

  function setMode(next: SearchMode) {
    void prefsStore.update({ globalSearchMode: next });
  }

  async function triggerSearch() {
    const generation = ++searchGeneration;
    const trimmed = debouncedQuery.value.trim();
    if (trimmed === '') {
      groups.value = [];
      isSearching.value = false;
      return;
    }
    const results = await ipc.search(trimmed, mode.value, projectContext());
    if (generation !== searchGeneration) return; // a newer search has since superseded this one
    groups.value = results.groups;
    isSearching.value = false;
  }

  // §6: "Selection index resets to 0 on every query change." §8: typing debounce resets on every
  // keystroke; `groups` (so `results`) is deliberately left untouched here -- stale results stay
  // on screen through the debounce window, only replaced once a new search actually completes.
  watch(query, (q) => {
    selectedIndex.value = 0;
    if (q.trim() !== '') isSearching.value = true;
    if (debounceTimer !== undefined) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      debouncedQuery.value = q;
    }, TYPING_DEBOUNCE_MS);
  });

  watch(debouncedQuery, () => void triggerSearch());
  // §8: "a mode switch re-searches immediately with the current debouncedQuery" -- not debounced.
  watch(mode, () => void triggerSearch());

  function navigateToScreenActivation(screen: string): SearchActivation {
    const item = navigationItems.find((i) => i.id === screen);
    return item ? { kind: 'route', to: item.to } : { kind: 'none' };
  }

  /** §5's table, minus the two quick actions (no `SearchAction` IPC variant exists for them,
   * §5's own "No IPC equivalent today" -- handled separately via `quickActions` above). */
  function resolveAction(action: SearchAction): SearchActivation {
    switch (action.kind) {
      case 'openSession':
        return { kind: 'session', projectId: action.projectId, sessionId: action.sessionId };
      case 'openDebugLog':
        return { kind: 'session', projectId: action.projectId, sessionId: action.sessionId, debugLog: true };
      case 'navigateTo':
        return navigateToScreenActivation(action.screen);
      case 'openDetail':
        // The generic /config/:kind/:scope/:slug route only exists for these four slug-
        // addressable kinds (routes.ts's own comment); Plans/Memory degrade to a screen-level
        // navigation -- their detail pages are placeholders today regardless (M3-03/M3-04
        // pending), so there is nothing deeper to link to yet.
        if (action.ref && (['command', 'skill', 'outputStyle', 'agent'] as const).includes(action.ref.kind as never)) {
          return {
            kind: 'route',
            to: { name: 'config-detail', params: { kind: action.ref.kind, scope: action.ref.project ? 'project' : 'global', slug: action.ref.slug } },
          };
        }
        return navigateToScreenActivation(action.screen);
    }
  }

  /** §3: Return/click on an empty-state row -- SCREENS rows first (`emptyScreens`), then RECENT. */
  function resolveEmptyRow(index: number): SearchActivation {
    if (index < emptyScreens.value.length) return { kind: 'route', to: emptyScreens.value[index].to };
    const recent = recentSessions.value[index - emptyScreens.value.length];
    return recent ? { kind: 'session', projectId: recent.projectId, sessionId: recent.session.id } : { kind: 'none' };
  }

  /** §6's Return handling: "If query.isEmpty: navigateEmptyState(at: selectedIndex). Else:
   * navigate(to: flatResults[selectedIndex]) ... otherwise ignored." */
  function activate(): SearchActivation {
    if (isEmptyQuery.value) return resolveEmptyRow(selectedIndex.value);
    const hit = flatResults.value[selectedIndex.value];
    return hit ? resolveAction(hit.action) : { kind: 'none' };
  }

  function moveSelection(delta: number) {
    const max = (isEmptyQuery.value ? emptyScreens.value.length + recentSessions.value.length : flatResults.value.length) - 1;
    if (max < 0) return;
    selectedIndex.value = Math.max(0, Math.min(selectedIndex.value + delta, max));
  }

  return {
    isOpen,
    query,
    debouncedQuery,
    isSearching,
    selectedIndex,
    groups,
    recentSessions,
    sidebarCounts,
    mode,
    emptyScreens,
    flatResults,
    isEmptyQuery,
    quickActions,
    open,
    close,
    toggleOpen,
    setMode,
    activate,
    activateQuickAction,
    moveSelection,
  };
});
