// AC: "Spec covers truncation at 80 lines, show-all, `parseMarkdownInResults`, `wrapCodeLines`
// and copy." `parseMarkdownInResults` is `SystemContentView`'s own preference (M2-17) -- checked
// directly, `ToolBlockView.swift` never reads it at all. The real markdown/raw toggle here is a
// local `showRawContent` state (`markdownToggleButton`), not a persisted preference; this spec
// tests that real mechanism instead of inventing a binding to a pref the real component never
// reads, with the discrepancy recorded honestly in the task's own progress-log entry.
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs, ToolResult, ToolUse } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import ToolBlock from './ToolBlock.vue';

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

function makeTool(overrides: Partial<ToolUse> = {}): ToolUse {
  return { id: 't1', toolUseId: 'tu1', name: 'Read', inputPretty: '{}', command: null, filePath: null, edit: null, ...overrides };
}

function makeResult(overrides: Partial<ToolResult> = {}): ToolResult {
  return { id: 'r1', toolUseId: 'tu1', content: 'some output', isError: false, ...overrides };
}

function longText(lineCount: number): string {
  return Array.from({ length: lineCount }, (_, i) => `line ${i}`).join('\n');
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountBlock(tool: ToolUse, result: ToolResult | null, prefsOverrides: Partial<Prefs> = {}) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs(prefsOverrides);
  mockIPC((cmd, args) => {
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      const current = usePrefsStore().prefs!;
      usePrefsStore().prefs = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return usePrefsStore().prefs;
    }
    if (cmd === 'diff_lines') return { lines: [], truncated: false, maxLineNumber: 0 };
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(ToolBlock, { props: { tool, result }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
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

describe('ToolBlock: truncation at 80 lines', () => {
  it('shows only the first 80 lines and a "Show all" toggle when content is longer', async () => {
    const wrapper = await mountBlock(makeTool(), makeResult({ content: longText(100) }), { autoExpandBlocks: true });
    const shown = wrapper.find('pre').text().split('\n');
    expect(shown.length).toBe(80);
    expect(shown[0]).toBe('line 0');
    expect(shown[79]).toBe('line 79');
    expect(wrapper.text()).toContain('Show all 100 lines');
  });

  it('"Show all" reveals every line; clicking again re-truncates', async () => {
    const wrapper = await mountBlock(makeTool(), makeResult({ content: longText(100) }), { autoExpandBlocks: true });

    await wrapper.find('button[title="Show all 100 lines"]').trigger('click');
    expect(wrapper.find('pre').text().split('\n').length).toBe(100);

    await wrapper.find('button[title="Show less"]').trigger('click');
    expect(wrapper.find('pre').text().split('\n').length).toBe(80);
  });

  it('content at or under 80 lines has no truncation toggle', async () => {
    const wrapper = await mountBlock(makeTool(), makeResult({ content: longText(80) }), { autoExpandBlocks: true });
    expect(wrapper.find('pre').text().split('\n').length).toBe(80);
    expect(wrapper.text()).not.toContain('Show all');
  });
});

describe('ToolBlock: markdown/raw toggle (the real mechanism behind the AC\'s "parseMarkdownInResults" wording)', () => {
  it('a non-bash tool defaults to raw text, not rendered markdown', async () => {
    const wrapper = await mountBlock(makeTool({ name: 'Read' }), makeResult({ content: '# heading' }), { autoExpandBlocks: true });
    expect(wrapper.find('pre').exists()).toBe(true);
    expect(wrapper.find('.markdown-view').exists()).toBe(false);
  });

  it('the toggle button switches a non-bash tool to rendered markdown and back', async () => {
    const wrapper = await mountBlock(makeTool({ name: 'Read' }), makeResult({ content: '# heading' }), { autoExpandBlocks: true });

    await wrapper.find('button[title="Show rendered markdown"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('.markdown-view').exists()).toBe(true);
    expect(wrapper.find('pre').exists()).toBe(false);

    await wrapper.find('button[title="Show raw content"]').trigger('click');
    expect(wrapper.find('pre').exists()).toBe(true);
  });

  it('a Bash tool has no markdown toggle at all -- always raw', async () => {
    const wrapper = await mountBlock(makeTool({ name: 'Bash', command: 'ls -la' }), makeResult({ content: 'file1\nfile2' }), { autoExpandBlocks: true });
    expect(wrapper.find('button[title="Show rendered markdown"]').exists()).toBe(false);
    expect(wrapper.find('pre').exists()).toBe(true);
  });
});

describe('ToolBlock: wrapCodeLines', () => {
  it('the wrap-toggle button flips the shared wrapCodeLines pref through a real update_prefs round trip', async () => {
    const wrapper = await mountBlock(makeTool(), makeResult(), { autoExpandBlocks: true, wrapCodeLines: true });
    expect(wrapper.find('pre').classes()).toContain('whitespace-pre-wrap');

    await wrapper.find('button[title="Disable line wrapping"]').trigger('click');
    await flushPromises();

    expect(usePrefsStore().prefs?.wrapCodeLines).toBe(false);
    expect(wrapper.find('pre').classes()).toContain('whitespace-pre');
  });
});

describe('ToolBlock: copy', () => {
  it('copies the full, untruncated result content', async () => {
    vi.useFakeTimers();
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = await mountBlock(makeTool(), makeResult({ content: longText(100) }), { autoExpandBlocks: true });

    await wrapper.find('button[title="Copy content"]').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith(longText(100));
    expect(wrapper.html()).toContain('lucide-check');

    await vi.advanceTimersByTimeAsync(1500);
    expect(wrapper.html()).toContain('lucide-files');
  });

  it('copies the bash command, separately from the output content', async () => {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = await mountBlock(makeTool({ name: 'Bash', command: 'rm -rf /tmp/x' }), makeResult({ content: 'done' }), { autoExpandBlocks: true });

    await wrapper.find('button[title="Copy command"]').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith('rm -rf /tmp/x');
  });

  it('copy-as-Markdown includes the tool name, the command, and the output', async () => {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = await mountBlock(makeTool({ name: 'Bash', command: 'echo hi' }), makeResult({ content: 'hi' }), { autoExpandBlocks: true });

    await wrapper.find('button[title="Copy as Markdown"]').trigger('click');
    await flushPromises();
    const [copiedMarkdown] = vi.mocked(writeText).mock.calls.at(-1)!;
    expect(copiedMarkdown).toContain('### Tool: Bash');
    expect(copiedMarkdown).toContain('echo hi');
    expect(copiedMarkdown).toContain('**Output:**');
    expect(copiedMarkdown).toContain('hi');
  });
});

describe('ToolBlock: expand/collapse and status', () => {
  it('starts collapsed when autoExpandBlocks is off, expands on click', async () => {
    const wrapper = await mountBlock(makeTool(), makeResult(), { autoExpandBlocks: false });
    expect(wrapper.find('pre').exists()).toBe(false);

    await wrapper.find('button[title="Expand"]').trigger('click');
    expect(wrapper.find('pre').exists()).toBe(true);
  });

  it('shows "No output" when the result has no content', async () => {
    const wrapper = await mountBlock(makeTool(), makeResult({ content: '' }), { autoExpandBlocks: true });
    expect(wrapper.text()).toContain('No output');
  });

  it('renders EditDiff instead of the result content when the tool carries edit data', async () => {
    const wrapper = await mountBlock(makeTool({ name: 'Edit', edit: { oldString: 'a', newString: 'b' } }), makeResult(), { autoExpandBlocks: true });
    expect(wrapper.find('button[title="Copy diff text"]').exists()).toBe(true);
    expect(wrapper.find('pre').exists()).toBe(false);
  });

  it('a Bash tool badge reads Exit 0/Exit 1; any other tool reads Done/Error', async () => {
    const bashOk = await mountBlock(makeTool({ name: 'Bash' }), makeResult({ isError: false }));
    expect(bashOk.text()).toContain('Exit 0');

    const readError = await mountBlock(makeTool({ name: 'Read' }), makeResult({ isError: true }));
    expect(readError.text()).toContain('Error');
  });
});
