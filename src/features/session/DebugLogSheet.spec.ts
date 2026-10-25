// M3-08's own AC: "Paging uses get_debug_log_page; the level filter covers DEBUG, WARN and
// ERROR." Ported from DebugLogView.swift. Every query goes through `document.body`, not
// `wrapper.find()` -- this component's root is a `<Teleport to="body">` (SearchOverlay.spec.ts's
// own documented reasoning: Vue Test Utils does not search teleported content from the mounting
// wrapper).
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DebugLogEntry, DebugLogPage } from '@/ipc/bindings';
import { events } from '@/ipc/bindings';
import { useSessionStore } from './store';
import DebugLogSheet from './DebugLogSheet.vue';

const writeTextMock = vi.fn().mockResolvedValue(undefined);
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: (...args: unknown[]) => writeTextMock(...args) }));

function body() {
  return new DOMWrapper(document.body);
}

function makeEntry(overrides: Partial<DebugLogEntry> = {}): DebugLogEntry {
  return { id: 0, timestamp: '2026-01-01T12:00:05.250Z', level: 'debug', message: 'hello', ...overrides };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountSheet(ipcHandler: (cmd: string, args: unknown) => unknown) {
  mockIPC((cmd, args) => {
    if (ipcHandler) {
      const result = ipcHandler(cmd, args);
      if (result !== undefined) return result;
    }
    throw new Error(`unexpected command: ${cmd}`);
  }, { shouldMockEvents: true });
  useSessionStore().showDebugLogSessionId = 's1';
  const wrapper = mount(DebugLogSheet, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
  setActivePinia(createPinia());
  writeTextMock.mockClear();
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('DebugLogSheet: initial load (AC: "Paging uses get_debug_log_page")', () => {
  it('fetches the first page with no level filter', async () => {
    let args: unknown = null;
    await mountSheet((cmd, a) => {
      if (cmd === 'get_debug_log_page') {
        args = a;
        return { entries: [makeEntry()], offset: 0, total: 1 } satisfies DebugLogPage;
      }
    });
    expect(args).toEqual({ sessionId: 's1', offset: 0, limit: 200, level: null });
    expect(body().text()).toContain('hello');
    expect(body().text()).toContain('1 entry');
  });

  it('shows "No debug log found" when the session has no log', async () => {
    await mountSheet(() => ({ entries: [], offset: 0, total: 0 }) satisfies DebugLogPage);
    expect(body().text()).toContain('No debug log found');
  });
});

describe('DebugLogSheet: level filter (AC: "the level filter covers DEBUG, WARN and ERROR")', () => {
  it('clicking a level chip re-fetches from offset 0 with that level', async () => {
    const calls: unknown[] = [];
    await mountSheet((cmd, a) => {
      if (cmd === 'get_debug_log_page') {
        calls.push(a);
        return { entries: [makeEntry({ level: 'warn' })], offset: 0, total: 1 } satisfies DebugLogPage;
      }
    });
    calls.length = 0;

    const warnChip = body().findAll('[role="radio"]').find((b) => b.text() === 'WARN');
    await warnChip!.trigger('click');
    await flushPromises();

    expect(calls).toEqual([{ sessionId: 's1', offset: 0, limit: 200, level: 'warn' }]);
  });

  it('switching back to "All" re-fetches with level: null', async () => {
    const calls: unknown[] = [];
    await mountSheet((cmd, a) => {
      if (cmd === 'get_debug_log_page') {
        calls.push(a);
        return { entries: [makeEntry()], offset: 0, total: 1 } satisfies DebugLogPage;
      }
    });
    const errorChip = body().findAll('[role="radio"]').find((b) => b.text() === 'ERROR');
    await errorChip!.trigger('click');
    await flushPromises();
    calls.length = 0;

    const allChip = body().findAll('[role="radio"]').find((b) => b.text() === 'All');
    await allChip!.trigger('click');
    await flushPromises();

    expect(calls).toEqual([{ sessionId: 's1', offset: 0, limit: 200, level: null }]);
  });
});

describe('DebugLogSheet: search (client-side, no refetch)', () => {
  it('filters the already-loaded entries without calling get_debug_log_page again', async () => {
    const calls: unknown[] = [];
    await mountSheet((cmd, a) => {
      if (cmd === 'get_debug_log_page') {
        calls.push(a);
        return { entries: [makeEntry({ id: 1, message: 'needle here' }), makeEntry({ id: 2, message: 'nothing relevant' })], offset: 0, total: 2 } satisfies DebugLogPage;
      }
    });
    expect(calls).toHaveLength(1);

    await body().find('input[aria-label="Filter log entries"]').setValue('needle');

    expect(body().text()).toContain('needle here');
    expect(body().text()).not.toContain('nothing relevant');
    expect(calls).toHaveLength(1); // still just the one initial fetch
  });

  it('shows "No matching entries" when the search excludes every loaded entry', async () => {
    await mountSheet((cmd) => {
      if (cmd === 'get_debug_log_page') return { entries: [makeEntry({ message: 'foo' })], offset: 0, total: 1 } satisfies DebugLogPage;
    });

    await body().find('input[aria-label="Filter log entries"]').setValue('zzz-no-match');

    expect(body().text()).toContain('No matching entries');
  });
});

describe('DebugLogSheet: paging (scroll near the bottom loads more)', () => {
  it('loads the next page at the right offset, keeping the active level', async () => {
    const calls: unknown[] = [];
    await mountSheet((cmd, a) => {
      if (cmd === 'get_debug_log_page') {
        calls.push(a);
        const { offset } = a as { offset: number };
        if (offset === 0) return { entries: [makeEntry({ id: 0 })], offset: 0, total: 2 } satisfies DebugLogPage;
        return { entries: [makeEntry({ id: 1 })], offset, total: 2 } satisfies DebugLogPage;
      }
    });
    calls.length = 0;

    const scrollArea = body().find('[class*="overflow-y-auto"]');
    Object.defineProperty(scrollArea.element, 'scrollHeight', { configurable: true, value: 1000 });
    Object.defineProperty(scrollArea.element, 'clientHeight', { configurable: true, value: 500 });
    scrollArea.element.scrollTop = 900; // 1000 - 900 - 500 < 200
    await scrollArea.trigger('scroll');
    await flushPromises();

    expect(calls).toEqual([{ sessionId: 's1', offset: 1, limit: 200, level: null }]);
  });
});

describe('DebugLogSheet: copy (always the full, unfiltered log)', () => {
  it('copies every entry regardless of the active level filter', async () => {
    await mountSheet((cmd, a) => {
      if (cmd === 'get_debug_log_page') {
        const { level, limit } = a as { level: string | null; limit: number };
        if (limit === 200) return { entries: [makeEntry({ level: (level as 'warn') ?? 'debug' })], offset: 0, total: 1 } satisfies DebugLogPage;
        // The copy button's own fetch: unfiltered, with the "whole file" limit.
        expect(level).toBeNull();
        return { entries: [makeEntry({ message: 'full log line' })], offset: 0, total: 1 } satisfies DebugLogPage;
      }
    });
    const warnChip = body().findAll('[role="radio"]').find((b) => b.text() === 'WARN');
    await warnChip!.trigger('click');
    await flushPromises();

    await body().find('button[title="Copy Log"]').trigger('click');
    await flushPromises();

    expect(writeTextMock).toHaveBeenCalledWith(expect.stringContaining('full log line'));
  });
});

describe('DebugLogSheet: refreshes on data-changed', () => {
  it('a real data-changed event for "debugLogs" triggers a reload', async () => {
    let callCount = 0;
    await mountSheet((cmd) => {
      if (cmd === 'get_debug_log_page') {
        callCount += 1;
        return { entries: [makeEntry()], offset: 0, total: 1 } satisfies DebugLogPage;
      }
    });
    expect(callCount).toBe(1);

    await events.dataChanged.emit([{ area: 'debugLogs', key: null }]);
    await flushPromises();

    expect(callCount).toBe(2);
  });

  it('an unrelated area does not trigger a reload', async () => {
    let callCount = 0;
    await mountSheet((cmd) => {
      if (cmd === 'get_debug_log_page') {
        callCount += 1;
        return { entries: [makeEntry()], offset: 0, total: 1 } satisfies DebugLogPage;
      }
    });
    callCount = 0;

    await events.dataChanged.emit([{ area: 'todos', key: null }]);
    await flushPromises();

    expect(callCount).toBe(0);
  });
});

describe('DebugLogSheet: close', () => {
  it('clicking Close clears showDebugLogSessionId', async () => {
    await mountSheet(() => ({ entries: [], offset: 0, total: 0 }) satisfies DebugLogPage);

    await body().find('button[title="Close"]').trigger('click');

    expect(useSessionStore().showDebugLogSessionId).toBeNull();
  });
});
