// AC: "The copied resume command is `claude --resume <sessionId>`." / "Terminal and reveal
// buttons come later (M3-55)."
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { SessionSummary } from '@/ipc/bindings';
import { useSessionStore } from './store';
import SessionToolbar from './SessionToolbar.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function makeSummary(overrides: Partial<SessionSummary> = {}): SessionSummary {
  return {
    id: 'sess-123',
    projectId: 'proj-1',
    title: 'Test session',
    preview: null,
    startedAt: new Date().toISOString(),
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

function mountToolbar(summary: SessionSummary) {
  setActivePinia(createPinia());
  const wrapper = mount(SessionToolbar, { props: { summary }, attachTo: document.body });
  mounted.push(wrapper);
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

describe('SessionToolbar: Resume (AC)', () => {
  it('copies exactly "claude --resume <sessionId>"', async () => {
    vi.useFakeTimers();
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = mountToolbar(makeSummary({ id: 'abc-999' }));

    await wrapper.find('button[title="Resume"]').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith('claude --resume abc-999');
    expect(wrapper.text()).toContain('Copied');

    await vi.advanceTimersByTimeAsync(4000);
    expect(wrapper.text()).toContain('Resume');
    expect(wrapper.text()).not.toContain('Copied');
  });
});

describe('SessionToolbar: no terminal/reveal buttons (AC2)', () => {
  it('has no "Show in Finder"/reveal button and no terminal-launch side effect', async () => {
    const wrapper = mountToolbar(makeSummary());
    expect(wrapper.find('button[title="Show in Finder"]').exists()).toBe(false);
    expect(wrapper.text()).not.toContain('Terminal');
  });
});

describe('SessionToolbar: copy session id', () => {
  it('copies summary.id directly, not a file name or path', async () => {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = mountToolbar(makeSummary({ id: 'the-session-id' }));
    await wrapper.find('button[title="Copy session ID"]').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith('the-session-id');
  });
});

describe('SessionToolbar: file history toggle', () => {
  it('is disabled when the session has no file history, enabled and toggling the store when it does', async () => {
    const disabled = mountToolbar(makeSummary({ hasFileHistory: false }));
    expect((disabled.find('button[title="File History"]').element as HTMLButtonElement).disabled).toBe(true);

    const enabled = mountToolbar(makeSummary({ hasFileHistory: true }));
    expect((enabled.find('button[title="File History"]').element as HTMLButtonElement).disabled).toBe(false);
    await enabled.find('button[title="File History"]').trigger('click');
    expect(useSessionStore().isShowingFileHistory).toBe(true);
  });
});

describe('SessionToolbar: debug log opener', () => {
  it('shows the button only when a log exists, and sets showDebugLogSessionId on click', async () => {
    const noLog = mountToolbar(makeSummary({ hasDebugLog: false }));
    expect(noLog.find('button[title="Debug Log"]').exists()).toBe(false);

    const withLog = mountToolbar(makeSummary({ id: 'sess-with-log', hasDebugLog: true }));
    await withLog.find('button[title="Debug Log"]').trigger('click');
    expect(useSessionStore().showDebugLogSessionId).toBe('sess-with-log');
  });
});

describe('SessionToolbar: export', () => {
  it('sets isShowingExportOptions on click', async () => {
    const wrapper = mountToolbar(makeSummary());
    await wrapper.find('button[title="Export Session"]').trigger('click');
    expect(useSessionStore().isShowingExportOptions).toBe(true);
  });
});

describe('SessionToolbar: expand all', () => {
  it('toggles sessionStore.allBlocksExpanded and its own label', async () => {
    const wrapper = mountToolbar(makeSummary());
    expect(wrapper.find('button[title="Expand All"]').exists()).toBe(true);

    await wrapper.find('button[title="Expand All"]').trigger('click');
    expect(useSessionStore().allBlocksExpanded).toBe(true);
    expect(wrapper.find('button[title="Collapse All"]').exists()).toBe(true);

    await wrapper.find('button[title="Collapse All"]').trigger('click');
    expect(useSessionStore().allBlocksExpanded).toBe(false);
  });
});

describe('SessionToolbar: search field', () => {
  it('binds sessionStore.sessionSearchQuery and sets isSessionSearchActive accordingly', async () => {
    const wrapper = mountToolbar(makeSummary());
    await wrapper.find('input').setValue('needle');
    expect(useSessionStore().sessionSearchQuery).toBe('needle');
    expect(useSessionStore().isSessionSearchActive).toBe(true);

    await wrapper.find('button[aria-label="Clear search"]').trigger('click');
    expect(useSessionStore().sessionSearchQuery).toBe('');
    expect(useSessionStore().isSessionSearchActive).toBe(false);
  });
});
