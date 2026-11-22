// AC: "The copied resume command is `claude --resume <sessionId>`." (M2-22) The terminal and reveal
// buttons it left for M3-55 are covered in the "open a terminal and show the file" block.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs, SessionSummary, TargetOs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { usePrefsStore } from '@/stores/prefs';
import { useToastsStore } from '@/stores/toasts';
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

// M3-55. The commands take ids, never paths (ADR 0010), so the assertions are on the exact target.
describe('SessionToolbar: open a terminal and show the file (M3-55)', () => {
  let launched: [string, unknown][];

  function prefs(overrides: Partial<Prefs> = {}): Prefs {
    return {
      schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1, sessionLayout: 'grid',
      configProjectPath: null, configLayouts: {}, showAgentSessions: false, textEditor: 'vscode', preferredTerminal: 'terminal',
      openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true, autoExpandBlocks: true,
      parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [], ...overrides,
    };
  }

  function mountLaunching(options: { os?: TargetOs | null; prefs?: Partial<Prefs>; fail?: string } = {}) {
    setActivePinia(createPinia());
    usePlatformStore().os = options.os === undefined ? 'macos' : options.os;
    usePrefsStore().prefs = prefs(options.prefs);
    launched = [];
    mockIPC((cmd, args) => {
      if (cmd === 'open_terminal' || cmd === 'reveal_in_file_manager') {
        launched.push([cmd, args]);
        if (options.fail) throw { kind: 'notFound', message: options.fail, path: null };
        return null;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    const wrapper = mount(SessionToolbar, { props: { summary: makeSummary({ id: 'abc-999', projectId: 'proj-7' }) }, attachTo: document.body });
    mounted.push(wrapper);
    return wrapper;
  }

  describe('Resume with openTerminalOnBash', () => {
    it('copies the literal command and opens the preferred terminal at the session when enabled', async () => {
      const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
      const wrapper = mountLaunching({ prefs: { openTerminalOnBash: true, preferredTerminal: 'iterm2' } });
      const button = wrapper.find('button[title="Resume (copy and open iTerm2)"]');
      expect(button.exists()).toBe(true);
      await button.trigger('click');
      await flushPromises();
      expect(writeText).toHaveBeenLastCalledWith('claude --resume abc-999');
      expect(launched).toEqual([['open_terminal', { projectId: 'proj-7', sessionId: 'abc-999' }]]);
    });

    it('only copies when the preference is off', async () => {
      const wrapper = mountLaunching({ prefs: { openTerminalOnBash: false } });
      await wrapper.find('button[title="Resume"]').trigger('click');
      await flushPromises();
      expect(launched).toEqual([]);
    });

    it('only copies when the saved terminal does not exist on this OS (AC2), and does not promise one', async () => {
      const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
      const wrapper = mountLaunching({ os: 'windows', prefs: { openTerminalOnBash: true, preferredTerminal: 'iterm2' } });
      await wrapper.find('button[title="Resume"]').trigger('click');
      await flushPromises();
      expect(writeText).toHaveBeenLastCalledWith('claude --resume abc-999');
      expect(launched).toEqual([]);
    });

    it('opens nothing until the OS is known', async () => {
      const wrapper = mountLaunching({ os: null, prefs: { openTerminalOnBash: true } });
      await wrapper.find('button[title="Resume"]').trigger('click');
      await flushPromises();
      expect(launched).toEqual([]);
    });

    it('still copies, and shows why, when the terminal cannot be opened', async () => {
      const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
      const wrapper = mountLaunching({ fail: 'no such project', prefs: { openTerminalOnBash: true } });
      await wrapper.find('button[title^="Resume ("]').trigger('click');
      await flushPromises();
      expect(writeText).toHaveBeenLastCalledWith('claude --resume abc-999');
      expect(useToastsStore().current).toMatchObject({ message: 'no such project', style: 'error' });
    });

    it('is not triggered by copying the session id', async () => {
      const wrapper = mountLaunching({ prefs: { openTerminalOnBash: true } });
      await wrapper.find('button[title="Copy session ID"]').trigger('click');
      await flushPromises();
      expect(launched).toEqual([]);
    });
  });

  describe('the show-the-file button', () => {
    it('reveals this session by ids, never a path', async () => {
      const wrapper = mountLaunching();
      await wrapper.find('button[title="Show in Finder"]').trigger('click');
      await flushPromises();
      expect(launched).toEqual([['reveal_in_file_manager', { target: { kind: 'session', projectId: 'proj-7', sessionId: 'abc-999' } }]]);
    });

    it.each<[TargetOs | null, string]>([
      ['macos', 'Finder'],
      ['windows', 'File Explorer'],
      ['linux', 'file manager'],
      [null, 'file manager'],
    ])('is named for what the OS calls it (%s: %s)', (os, name) => {
      const wrapper = mountLaunching({ os });
      const button = wrapper.find(`button[title="Show in ${name}"]`);
      expect(button.exists()).toBe(true);
      expect(button.attributes('aria-label')).toBe(`Show in ${name}`);
    });

    it('is always offered: every OS has a file manager', () => {
      expect(mountLaunching({ os: 'linux', prefs: { textEditor: 'xcode', preferredTerminal: 'iterm2' } }).find('button[title="Show in file manager"]').exists()).toBe(true);
    });

    it('shows why when it cannot be opened', async () => {
      const wrapper = mountLaunching({ fail: 'the session file is gone' });
      await wrapper.find('button[title="Show in Finder"]').trigger('click');
      await flushPromises();
      expect(useToastsStore().current).toMatchObject({ message: 'the session file is gone', style: 'error' });
    });
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
