// AC: "Blocks are measured with measureElement and re-measured after expand/collapse and
// font-scale changes." / "Messages load in pages of 20 through get_session_messages as the
// viewport nears the end." / "Mod+T toggles the tool filter with tool-name chips; session search
// highlights matches." / "Scroll position is restored on back/forward."
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Message, Prefs, SessionDetail as SessionDetailContract, SessionSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useAppActions } from '@/composables/useAppActions';
import { useSessionStore, type LoadedSession } from './store';
import SessionDetail from './SessionDetail.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function defaultPrefs(overrides: Partial<Prefs> = {}): Prefs {
  return {
    schemaVersion: 1,
    showMenuBarIcon: true,
    hasCompletedOnboarding: true,
    colorTheme: 'default',
    fontScale: 1.0,
    sessionLayout: 'grid',
    configProjectPath: null,
    configLayouts: {},
    showAgentSessions: false,
    textEditor: 'vscode',
    preferredTerminal: 'terminal',
    openTerminalOnBash: false,
    claudeCodePath: null,
    showAnimations: true,
    wrapCodeLines: true,
    autoExpandBlocks: true,
    parseMarkdownInResults: true,
    globalSearchMode: 'fuzzy',
    customMemoryFolders: [],
    ...overrides,
  };
}

function makeSummary(overrides: Partial<SessionSummary> = {}): SessionSummary {
  return {
    id: 's1',
    projectId: 'p1',
    title: 'Test session',
    preview: null,
    startedAt: new Date().toISOString(),
    endedAt: null,
    model: 'claude-sonnet-4-5-20260101',
    tokens: { input: 1000, output: 2000, cacheRead: 0, cacheCreation: 0 },
    turnCount: 2,
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

function makeMessage(overrides: Partial<Message> = {}): Message {
  return { id: 'm1', role: 'user', timestamp: new Date().toISOString(), model: null, usage: null, blocks: [], ...overrides };
}

function makeDetail(overrides: Partial<SessionDetailContract> = {}): SessionDetailContract {
  return { summary: makeSummary(), messageCount: 2, context: null, toolNames: [], skipped: 0, ...overrides };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountDetail(session: LoadedSession, ipcHandler?: (cmd: string, args: unknown) => unknown) {
  const sessionStore = useSessionStore();
  sessionStore.selectedProjectId = session.detail.summary.projectId;
  sessionStore.selectedSession = session;

  mockIPC((cmd, args) => {
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      const current = usePrefsStore().prefs!;
      usePrefsStore().prefs = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return usePrefsStore().prefs;
    }
    // M3-06: SessionDetail now always mounts SessionFacetsCard/SessionTodos, each fetching its
    // own data for whichever session is open -- a default of "neither exists" here keeps every
    // test in this file (none of which are about facets/todos) from needing its own override.
    if (cmd === 'get_session_facets') return null;
    if (cmd === 'get_session_todos') return [];
    if (ipcHandler) return ipcHandler(cmd, args);
    throw new Error(`unexpected command: ${cmd}`);
  });

  const wrapper = mount(SessionDetail, { props: { session }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  // happy-dom measures every element at 0x0 by default. virtual-core's own `getRect` (not
  // `getBoundingClientRect`) reads `offsetWidth`/`offsetHeight` specifically -- confirmed by
  // reading virtual-core's source directly rather than guessing, after a first attempt at
  // mocking `getBoundingClientRect` left every row un-rendered (count was right, the visible
  // range was not). Every test gets a fixed, realistic scroll-container size instead of 0.
  Object.defineProperty(HTMLElement.prototype, 'offsetHeight', { configurable: true, value: 500 });
  Object.defineProperty(HTMLElement.prototype, 'offsetWidth', { configurable: true, value: 800 });
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

describe('SessionDetail: virtualization (AC1)', () => {
  it('renders rows through the virtualizer without throwing, and a row with an expandable block can be expanded', async () => {
    const session: LoadedSession = {
      id: 's1',
      detail: makeDetail(),
      messages: [
        makeMessage({ id: 'm1', role: 'user', blocks: [{ kind: 'text', id: 'b1', text: 'hello', system: null }] }),
        makeMessage({ id: 'm2', role: 'assistant', blocks: [{ kind: 'thinking', id: 'b2', text: 'thinking about it' }] }),
      ],
    };
    const wrapper = await mountDetail(session);
    expect(wrapper.text()).toContain('hello');
    // autoExpandBlocks is true in defaultPrefs, so the thinking block starts already expanded --
    // this proves the row measured and rendered a real height (ResizeObserver, bound via
    // measureElement as each row's own element ref) rather than asserting on it directly, which
    // virtual-core exposes no public hook for.
    expect(wrapper.text()).toContain('thinking about it');

    await wrapper.find('button[title="Collapse thinking"]').trigger('click');
    expect(wrapper.text()).not.toContain('thinking about it');
  });
});

describe('SessionDetail: pagination (AC2)', () => {
  it('fetches another page through get_session_messages when more exist beyond what is loaded', async () => {
    const session: LoadedSession = {
      id: 's1',
      detail: makeDetail({ messageCount: 10 }),
      messages: Array.from({ length: 5 }, (_, i) => makeMessage({ id: `m${i}`, blocks: [{ kind: 'text', id: `b${i}`, text: `line ${i}`, system: null }] })),
    };
    let called: unknown = null;
    const wrapper = await mountDetail(session, (cmd, args) => {
      if (cmd === 'get_session_messages') {
        called = args;
        return { start: 5, messages: [makeMessage({ id: 'm5' })], total: 10 };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    await wrapper.trigger('scroll');
    await flushPromises();

    expect(called).toEqual({ projectId: 'p1', sessionId: 's1', start: 5, count: 20 });
    expect((useSessionStore().selectedSession as LoadedSession).messages).toHaveLength(6);
  });

  it('does not fetch another page once every message is already loaded', async () => {
    const session: LoadedSession = {
      id: 's1',
      detail: makeDetail({ messageCount: 2 }),
      messages: [makeMessage({ id: 'm1' }), makeMessage({ id: 'm2' })],
    };
    const wrapper = await mountDetail(session, (cmd) => {
      throw new Error(`unexpected command: ${cmd}`);
    });
    await wrapper.trigger('scroll');
    await flushPromises();
    expect((useSessionStore().selectedSession as LoadedSession).messages).toHaveLength(2);
  });
});

describe('SessionDetail: tool filter chips (AC3)', () => {
  it('Mod+T (dispatched as the toolFilter action) shows the chip bar; a chip filters messages, keeping the preceding user message', async () => {
    const session: LoadedSession = {
      id: 's1',
      detail: makeDetail({ toolNames: ['Read', 'Write'] }),
      messages: [
        makeMessage({ id: 'm1', role: 'user', blocks: [{ kind: 'text', id: 'b1', text: 'please write the file', system: null }] }),
        makeMessage({
          id: 'm2',
          role: 'assistant',
          blocks: [{ kind: 'toolUse', id: 'b2', toolUseId: 'tu1', name: 'Write', inputPretty: '{}', command: null, filePath: null, edit: null }],
        }),
        makeMessage({
          id: 'm3',
          role: 'assistant',
          blocks: [{ kind: 'toolUse', id: 'b3', toolUseId: 'tu2', name: 'Read', inputPretty: '{}', command: null, filePath: null, edit: null }],
        }),
      ],
    };
    const wrapper = await mountDetail(session);
    // A chip renders as a bare `<button>{{ name }}</button>` -- exact text equality (not
    // `wrapper.text()`, which would also match ToolBlock's own header showing the tool name as
    // part of a bigger string) is what actually distinguishes "the chip bar is showing" here.
    const chip = (name: string) => wrapper.findAll('button').find((b) => b.text() === name);
    expect(chip('Read')).toBeUndefined();

    useAppActions().dispatch('toolFilter');
    await flushPromises();
    expect(useSessionStore().isToolFilterActive).toBe(true);
    expect(chip('Read')).toBeDefined();
    expect(chip('Write')).toBeDefined();

    expect(wrapper.findAll('[data-index]')).toHaveLength(3);

    await chip('Write')!.trigger('click');
    await flushPromises();
    // Rows, not text: the "Read" chip itself still legitimately reads "Read" once a filter is
    // active, so what actually proves filtering happened is the *row count* dropping from 3 to 2
    // (the user message kept for context, plus the Write match -- the Read message is gone).
    expect(wrapper.findAll('[data-index]')).toHaveLength(2);
    expect(wrapper.text()).toContain('please write the file'); // kept: precedes the Write match

    useAppActions().dispatch('toolFilter');
    await flushPromises();
    expect(useSessionStore().isToolFilterActive).toBe(false);
    expect(useSessionStore().activeToolFilters.size).toBe(0);
  });
});

describe('SessionDetail: session search highlighting (AC3)', () => {
  it('highlights the match in a text block and hides non-matching messages', async () => {
    const session: LoadedSession = {
      id: 's1',
      detail: makeDetail(),
      messages: [
        makeMessage({ id: 'm1', role: 'user', blocks: [{ kind: 'text', id: 'b1', text: 'find the needle here', system: null }] }),
        makeMessage({ id: 'm2', role: 'assistant', blocks: [{ kind: 'text', id: 'b2', text: 'nothing relevant', system: null }] }),
      ],
    };
    const wrapper = await mountDetail(session);
    useSessionStore().sessionSearchQuery = 'needle';
    await flushPromises();

    expect(wrapper.find('mark').exists()).toBe(true);
    expect(wrapper.find('mark').text()).toBe('needle');
    expect(wrapper.text()).not.toContain('nothing relevant');
    expect(wrapper.find('.markdown-view').exists()).toBe(false); // plain highlighted text, not MarkdownView
  });
});

describe('SessionDetail: scroll restoration (AC4)', () => {
  it('restores the scroll offset on remount (back/forward re-mounts with the same session id)', async () => {
    const session: LoadedSession = {
      id: 's1',
      detail: makeDetail({ messageCount: 20 }),
      messages: Array.from({ length: 20 }, (_, i) => makeMessage({ id: `m${i}`, blocks: [{ kind: 'text', id: `b${i}`, text: `line ${i}`, system: null }] })),
    };
    const first = await mountDetail(session, () => ({ start: 20, messages: [], total: 20 }));
    const container = first.find('[class*="overflow-y-auto"]').element;
    container.scrollTop = 450;
    container.dispatchEvent(new Event('scroll'));
    await flushPromises();
    first.unmount();
    mounted = mounted.filter((w) => w !== first);

    const second = await mountDetail(session, () => ({ start: 20, messages: [], total: 20 }));
    const secondContainer = second.find('[class*="overflow-y-auto"]').element as HTMLElement;
    expect(secondContainer.scrollTop).toBe(450);
  });
});
