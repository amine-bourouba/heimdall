// Ported from HistoryEntry.swift:40-84 (date grouping) and HistoryListView.swift's own
// `filteredEntries`/`groupedEntries`/`loadHistory` (M3-02). Swift's `loader.loadAll()` loads
// every entry eagerly and only paginates the *display* of already-in-memory data -- Heimdall's
// own `list_history(offset, limit)` (M1-43) is a real paginated read instead, so search here can
// only filter what has been fetched so far, same limitation any paginated-plus-search UI has
// without a dedicated search command (none exists for history).
import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc } from '@/ipc/client';
import type { HistoryEntry } from '@/ipc/bindings';
import { fuzzyMatch } from '@/lib/fuzzyMatch';

/** `HistoryListView.swift:35`'s own `pageSize`. */
const PAGE_SIZE = 30;

export type HistoryDateGroup = 'today' | 'yesterday' | 'thisWeek' | 'lastWeek' | 'thisMonth' | 'older';

/** `HistoryDateGroup: CaseIterable`'s own declaration order (HistoryEntry.swift:42-47). */
export const HISTORY_DATE_GROUPS: HistoryDateGroup[] = ['today', 'yesterday', 'thisWeek', 'lastWeek', 'thisMonth', 'older'];

export const HISTORY_GROUP_TITLES: Record<HistoryDateGroup, string> = {
  today: 'Today',
  yesterday: 'Yesterday',
  thisWeek: 'This Week',
  lastWeek: 'Last Week',
  thisMonth: 'This Month',
  older: 'Older',
};

function startOfDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

// `Calendar.current.dateInterval(of: .weekOfYear, for:)` follows the user's own locale (en_US's
// own default, Sunday); a single fixed locale is already this port's own choice elsewhere
// (format.ts's `Intl.RelativeTimeFormat('en', ...)`), so the same default is used here rather
// than building a separate locale-detection story just for week boundaries.
function startOfWeek(d: Date): Date {
  const start = startOfDay(d);
  start.setDate(start.getDate() - start.getDay());
  return start;
}

function startOfMonth(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1);
}

/** `HistoryDateGroup.group(for:)` (HistoryEntry.swift:62-83), ported check for check, in the same
 * order -- each `else if` only runs once every earlier one has failed, so a date exactly on a
 * boundary (e.g. the first instant of this week) always resolves the same way Swift's own
 * `date >= weekStart` does. `now` defaults to the real clock but is always passed explicitly in
 * tests (the task's own AC: "spec with a fixed clock"), matching `formatRelativeTime`'s existing
 * convention (session/format.ts). */
export function groupForDate(iso: string, now: Date = new Date()): HistoryDateGroup {
  const date = new Date(iso);

  const today = startOfDay(now);
  if (date >= today) return 'today';

  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  if (date >= yesterday) return 'yesterday';

  const weekStart = startOfWeek(now);
  if (date >= weekStart) return 'thisWeek';

  const lastWeekStart = new Date(weekStart);
  lastWeekStart.setDate(lastWeekStart.getDate() - 7);
  if (date >= lastWeekStart) return 'lastWeek';

  const monthStart = startOfMonth(now);
  if (date >= monthStart) return 'thisMonth';

  return 'older';
}

/** `HistoryEntry.snippet` (HistoryEntry.swift:17-22): first 3 non-empty, trimmed lines, joined by
 * a space. */
export function historySnippet(display: string): string {
  return display
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line !== '')
    .slice(0, 3)
    .join(' ');
}

export const useHistoryStore = defineStore('history', () => {
  const entries = ref<HistoryEntry[]>([]);
  const total = ref(0);
  const skipped = ref(0);
  const isLoading = ref(true);
  const isLoadingMore = ref(false);
  const query = ref('');

  const hasMore = computed(() => entries.value.length < total.value);

  // Ported from `filteredEntries` (HistoryListView.swift:51-74), minus the project picker (not
  // in this task's own Outputs/AC list).
  const filteredEntries = computed<HistoryEntry[]>(() => {
    const q = query.value.trim();
    if (q === '') return entries.value;
    return entries.value
      .map((entry) => {
        const displayScore = fuzzyMatch(historySnippet(entry.display), q)?.score ?? 0;
        const projectScore = fuzzyMatch(projectName(entry), q)?.score ?? 0;
        return { entry, score: Math.max(displayScore, projectScore) };
      })
      .filter((r) => r.score > 0)
      .sort((a, b) => b.score - a.score)
      .map((r) => r.entry);
  });

  // Ported from `groupedEntries` (HistoryListView.swift:76-84): grouped by the fixed
  // HISTORY_DATE_GROUPS order, a group with no matches left out entirely.
  const groupedEntries = computed<{ group: HistoryDateGroup; entries: HistoryEntry[] }[]>(() => {
    const now = new Date();
    const buckets = new Map<HistoryDateGroup, HistoryEntry[]>();
    for (const entry of filteredEntries.value) {
      const group = groupForDate(entry.timestamp, now);
      const bucket = buckets.get(group);
      if (bucket) bucket.push(entry);
      else buckets.set(group, [entry]);
    }
    return HISTORY_DATE_GROUPS.flatMap((group) => {
      const items = buckets.get(group);
      return items ? [{ group, entries: items }] : [];
    });
  });

  async function load() {
    isLoading.value = true;
    const page = await ipc.listHistory(0, PAGE_SIZE);
    entries.value = page.entries;
    total.value = page.total;
    skipped.value = page.skipped;
    isLoading.value = false;
  }

  async function loadMore() {
    if (!hasMore.value || isLoadingMore.value) return;
    isLoadingMore.value = true;
    const page = await ipc.listHistory(entries.value.length, PAGE_SIZE);
    entries.value = [...entries.value, ...page.entries];
    total.value = page.total;
    skipped.value = page.skipped;
    isLoadingMore.value = false;
  }

  return { entries, total, skipped, isLoading, isLoadingMore, query, hasMore, filteredEntries, groupedEntries, load, loadMore };
});

/** `HistoryEntry.projectName` (HistoryEntry.swift:12-14): the last path component. */
export function projectName(entry: HistoryEntry): string {
  const parts = entry.project.split(/[/\\]/).filter((p) => p !== '');
  return parts.length > 0 ? parts[parts.length - 1] : entry.project;
}
