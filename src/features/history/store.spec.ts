// Carried over from HistoryEntryTests.swift (date-bucket cases re-verified against a fixed
// clock per this task's own AC, rather than HistoryEntryTests.swift's own `Date()`-relative
// cases, which are inherently non-deterministic) and HistoryDateGroupTests.swift.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { HistoryEntry, HistoryPage } from '@/ipc/bindings';
import { groupForDate, historySnippet, projectName, useHistoryStore } from './store';

// Sat Oct 24 2026, noon -- a fixed clock, picked and verified (not assumed) to fall on a
// Saturday, so startOfWeek(NOW) = Sun Oct 18 and every offset below lands in an unambiguous
// bucket regardless of which day of the week "now" happens to be.
const NOW = new Date(2026, 9, 24, 12, 0, 0);

function daysAgo(n: number): Date {
  const d = new Date(NOW);
  d.setDate(d.getDate() - n);
  return d;
}

function makeEntry(overrides: Partial<HistoryEntry> = {}): HistoryEntry {
  return { id: '1-0', display: 'a prompt', pastedContents: {}, timestamp: NOW.toISOString(), project: '/Users/test/project', ...overrides };
}

describe('groupForDate (HistoryEntry.swift:62-83, fixed clock)', () => {
  it('today: the same calendar day as now, even earlier in the day', () => {
    const earlierToday = new Date(NOW);
    earlierToday.setHours(0, 1, 0, 0);
    expect(groupForDate(earlierToday.toISOString(), NOW)).toBe('today');
  });

  it('yesterday: exactly one calendar day back', () => {
    expect(groupForDate(daysAgo(1).toISOString(), NOW)).toBe('yesterday');
  });

  it('thisWeek: within the current week but older than yesterday', () => {
    expect(groupForDate(daysAgo(3).toISOString(), NOW)).toBe('thisWeek'); // Oct 21, within [Oct 18, Oct 24)
  });

  it('lastWeek: the week immediately before this one', () => {
    expect(groupForDate(daysAgo(9).toISOString(), NOW)).toBe('lastWeek'); // Oct 15, within [Oct 11, Oct 18)
  });

  it('thisMonth: earlier in the same month, before last week started', () => {
    expect(groupForDate(daysAgo(20).toISOString(), NOW)).toBe('thisMonth'); // Oct 4, within [Oct 1, Oct 11)
  });

  it('older: a year ago', () => {
    const yearAgo = new Date(NOW);
    yearAgo.setFullYear(yearAgo.getFullYear() - 1);
    expect(groupForDate(yearAgo.toISOString(), NOW)).toBe('older');
  });

  it('is never ambiguous: every real-clock date resolves to exactly one of the 6 groups', () => {
    const group = groupForDate(new Date().toISOString());
    expect(['today', 'yesterday', 'thisWeek', 'lastWeek', 'thisMonth', 'older']).toContain(group);
  });
});

describe('historySnippet (HistoryEntry.swift:17-22)', () => {
  it('truncates to the first 3 non-empty lines, joined by a space', () => {
    expect(historySnippet('Line one\nLine two\nLine three\nLine four\nLine five')).toBe('Line one Line two Line three');
  });

  it('skips empty lines', () => {
    expect(historySnippet('First\n\n\nSecond')).toBe('First Second');
  });
});

describe('projectName (HistoryEntry.swift:12-14)', () => {
  it('extracts the last path component', () => {
    expect(projectName(makeEntry({ project: '/Users/test/Dev/my-project' }))).toBe('my-project');
  });
});

describe('history store: paging (AC2: "Paging uses list_history")', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('load() fetches the first page and hasMore reflects total vs. loaded count', async () => {
    const page: HistoryPage = { entries: [makeEntry({ id: 'a' }), makeEntry({ id: 'b' })], offset: 0, total: 5, skipped: 0 };
    mockIPC((cmd, args) => {
      expect(cmd).toBe('list_history');
      expect(args).toEqual({ offset: 0, limit: 30 });
      return page;
    });
    const store = useHistoryStore();

    await store.load();

    expect(store.entries.length).toBe(2);
    expect(store.total).toBe(5);
    expect(store.hasMore).toBe(true);
    expect(store.isLoading).toBe(false);
  });

  it('loadMore() appends the next page at the right offset', async () => {
    let calls = 0;
    mockIPC((_cmd, args) => {
      calls += 1;
      const a = args as { offset: number; limit: number };
      if (a.offset === 0) return { entries: [makeEntry({ id: 'a' })], offset: 0, total: 2, skipped: 0 };
      return { entries: [makeEntry({ id: 'b' })], offset: a.offset, total: 2, skipped: 0 };
    });
    const store = useHistoryStore();
    await store.load();

    await store.loadMore();

    expect(calls).toBe(2);
    expect(store.entries.map((e) => e.id)).toEqual(['a', 'b']);
    expect(store.hasMore).toBe(false);
  });

  it('loadMore() is a no-op once everything is loaded', async () => {
    let calls = 0;
    mockIPC(() => {
      calls += 1;
      return { entries: [makeEntry()], offset: 0, total: 1, skipped: 0 };
    });
    const store = useHistoryStore();
    await store.load();

    await store.loadMore();

    expect(calls).toBe(1); // the no-op loadMore() never called list_history again
  });
});

describe('history store: search and grouping (HistoryListView.swift:51-84)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    // Unlike the standalone groupForDate (tested above with an explicit `now`), the store's own
    // groupedEntries always calls the real `new Date()` -- confirmed the hard way: this test
    // fixture's "today" entry silently became "yesterday" the first time a real calendar day
    // passed since it was written. Faking the system clock, not just the fixture data, is what
    // actually fixes that instead of only deferring the same drift to the next day boundary.
    vi.useFakeTimers();
    vi.setSystemTime(NOW);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('filters by a fuzzy match on the prompt text or the project name', async () => {
    mockIPC(() => ({
      entries: [
        makeEntry({ id: 'match-display', display: 'deploy the service' }),
        makeEntry({ id: 'match-project', display: 'unrelated', project: '/Users/me/deploy-tool' }),
        makeEntry({ id: 'no-match', display: 'something else', project: '/Users/me/other' }),
      ],
      offset: 0,
      total: 3,
      skipped: 0,
    }));
    const store = useHistoryStore();
    await store.load();

    store.query = 'deploy';

    expect(store.filteredEntries.map((e) => e.id).sort()).toEqual(['match-display', 'match-project']);
  });

  it('groups into the fixed HISTORY_DATE_GROUPS order, omitting empty groups', async () => {
    mockIPC(() => ({
      entries: [makeEntry({ id: 'today-1', timestamp: NOW.toISOString() }), makeEntry({ id: 'older-1', timestamp: daysAgo(400).toISOString() })],
      offset: 0,
      total: 2,
      skipped: 0,
    }));
    const store = useHistoryStore();
    await store.load();

    expect(store.groupedEntries.map((g) => g.group)).toEqual(['today', 'older']); // thisWeek/lastWeek/thisMonth skipped: empty
  });
});

// M3-51. Both delete commands show a native confirmation (M3-46) and answer `cancelled` when it is
// declined; AC: "a `cancelled` result changes nothing in the UI".
describe('history store: deleting', () => {
  type Outcome = 'written' | 'cancelled' | 'unchanged';
  const page = (ids: string[], total = ids.length): HistoryPage => ({ entries: ids.map((id) => makeEntry({ id })), offset: 0, total, skipped: 0 });

  /** `lists` is what each successive list_history returns (the last repeats). */
  function mockHistory(outcome: Outcome, lists: HistoryPage[]) {
    const sent: [string, unknown][] = [];
    let listCalls = 0;
    mockIPC((cmd, args) => {
      sent.push([cmd, args]);
      if (cmd === 'list_history') return lists[Math.min(listCalls++, lists.length - 1)];
      if (cmd === 'delete_history_entry' || cmd === 'clear_history_older_than') return outcome;
      throw new Error(`unexpected command: ${cmd}`);
    });
    return sent;
  }

  it('deleteEntry sends the entry id and takes the entry out of the list and the count', async () => {
    const sent = mockHistory('written', [page(['a', 'b', 'c'])]);
    const store = useHistoryStore();
    await store.load();

    expect(await store.deleteEntry('b')).toBe('written');

    expect(sent).toContainEqual(['delete_history_entry', { entryId: 'b' }]);
    expect(store.entries.map((e) => e.id)).toEqual(['a', 'c']);
    expect(store.total).toBe(2);
  });

  it('deleteEntry: a declined confirmation changes nothing', async () => {
    mockHistory('cancelled', [page(['a', 'b', 'c'])]);
    const store = useHistoryStore();
    await store.load();

    expect(await store.deleteEntry('b')).toBe('cancelled');

    expect(store.entries.map((e) => e.id)).toEqual(['a', 'b', 'c']);
    expect(store.total).toBe(3);
  });

  it('deleteEntry: an entry that was already gone still leaves the list', async () => {
    mockHistory('unchanged', [page(['a', 'b'])]);
    const store = useHistoryStore();
    await store.load();

    expect(await store.deleteEntry('a')).toBe('unchanged');

    expect(store.entries.map((e) => e.id)).toEqual(['b']);
  });

  it('deleteEntry keeps the total honest when the list is only the first page', async () => {
    mockHistory('written', [page(['a', 'b'], 40)]);
    const store = useHistoryStore();
    await store.load();
    await store.deleteEntry('a');
    expect(store.total).toBe(39);
    expect(store.hasMore).toBe(true);
  });

  it('clearOlderThan sends the number of days and reads the list again without the skeleton', async () => {
    const sent = mockHistory('written', [page(['new', 'old']), page(['new'])]);
    const store = useHistoryStore();
    await store.load();

    expect(await store.clearOlderThan(30)).toBe('written');

    expect(sent).toContainEqual(['clear_history_older_than', { days: 30 }]);
    expect(store.entries.map((e) => e.id)).toEqual(['new']);
    expect(store.total).toBe(1);
    expect(store.isLoading).toBe(false);
  });

  it('clearOlderThan: a declined confirmation changes nothing and reads nothing again', async () => {
    const sent = mockHistory('cancelled', [page(['new', 'old']), page(['new'])]);
    const store = useHistoryStore();
    await store.load();

    expect(await store.clearOlderThan(30)).toBe('cancelled');

    expect(store.entries.map((e) => e.id)).toEqual(['new', 'old']);
    expect(sent.filter(([cmd]) => cmd === 'list_history')).toHaveLength(1);
  });

  it('clearOlderThan: when nothing was that old, nothing is re-read', async () => {
    const sent = mockHistory('unchanged', [page(['new'])]);
    const store = useHistoryStore();
    await store.load();

    expect(await store.clearOlderThan(180)).toBe('unchanged');

    expect(sent.filter(([cmd]) => cmd === 'list_history')).toHaveLength(1);
  });
});
