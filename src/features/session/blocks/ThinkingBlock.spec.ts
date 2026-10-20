// AC: "Spec covers truncation at 50 lines, expand, copy, and no animation under reduced motion."
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import ThinkingBlock from './ThinkingBlock.vue';

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
    autoExpandBlocks: false,
    parseMarkdownInResults: true,
    globalSearchMode: 'fuzzy',
    customMemoryFolders: [],
    ...overrides,
  };
}

let mounted: ReturnType<typeof mount>[] = [];

// `isExpanded` is set inside `onMounted`, matching the Swift reference app's own `.onAppear` read -- that is a
// real reactive change *after* the component's first render, not reflected in the wrapper until
// the next tick (confirmed for real: a first draft of this spec asserted immediately after
// `mount()` and saw the pre-onMounted, still-collapsed render). Every helper here awaits one.
async function mountBlock(text: string, prefsOverrides: Partial<Prefs> = {}) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs(prefsOverrides);
  mockIPC((cmd, args) => {
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      const current = usePrefsStore().prefs!;
      usePrefsStore().prefs = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return usePrefsStore().prefs;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(ThinkingBlock, { props: { text }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

function longText(lineCount: number): string {
  return Array.from({ length: lineCount }, (_, i) => `line ${i}`).join('\n');
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

describe('ThinkingBlock: collapsed by default, honours autoExpandBlocks once on mount', () => {
  it('starts collapsed when autoExpandBlocks is off', async () => {
    const wrapper = await mountBlock('short text', { autoExpandBlocks: false });
    expect(wrapper.find('pre').exists()).toBe(false);
  });

  it('starts expanded when autoExpandBlocks is on', async () => {
    const wrapper = await mountBlock('short text', { autoExpandBlocks: true });
    expect(wrapper.find('pre').exists()).toBe(true);
  });

  it('the header button toggles expansion', async () => {
    const wrapper = await mountBlock('short text', { autoExpandBlocks: false });
    await wrapper.find('button[title="Expand thinking"]').trigger('click');
    expect(wrapper.find('pre').exists()).toBe(true);

    await wrapper.find('button[title="Collapse thinking"]').trigger('click');
    expect(wrapper.find('pre').exists()).toBe(false);
  });
});

describe('ThinkingBlock: truncation at 50 lines', () => {
  it('shows only the first 50 lines and a "Show all" toggle when the text is longer', async () => {
    const wrapper = await mountBlock(longText(80), { autoExpandBlocks: true });
    const shown = wrapper.find('pre').text().split('\n');
    expect(shown.length).toBe(50);
    expect(shown[0]).toBe('line 0');
    expect(shown[49]).toBe('line 49');
    expect(wrapper.text()).toContain('Show all 80 lines');
  });

  it('clicking "Show all" reveals every line; clicking again re-truncates', async () => {
    const wrapper = await mountBlock(longText(80), { autoExpandBlocks: true });

    await wrapper.find('button[title="Show all 80 lines"]').trigger('click');
    expect(wrapper.find('pre').text().split('\n').length).toBe(80);
    expect(wrapper.text()).toContain('Show less');

    await wrapper.find('button[title="Show less"]').trigger('click');
    expect(wrapper.find('pre').text().split('\n').length).toBe(50);
  });

  it('a text at or under 50 lines has no truncation toggle at all', async () => {
    const wrapper = await mountBlock(longText(50), { autoExpandBlocks: true });
    expect(wrapper.find('pre').text().split('\n').length).toBe(50);
    expect(wrapper.text()).not.toContain('Show all');
  });
});

describe('ThinkingBlock: copy', () => {
  it('copies the full, untruncated text and shows a checkmark for 1.5s', async () => {
    vi.useFakeTimers();
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = await mountBlock(longText(80), { autoExpandBlocks: true });

    await wrapper.find('button[title="Copy thinking"]').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith(longText(80));
    expect(wrapper.html()).toContain('lucide-check');
    expect(wrapper.html()).not.toContain('lucide-files');

    await vi.advanceTimersByTimeAsync(1500);
    expect(wrapper.html()).toContain('lucide-files');
    expect(wrapper.html()).not.toContain('lucide-check');
  });
});

describe('ThinkingBlock: no animation under reduced motion', () => {
  it('the breathe effect class is absent while expanded under prefers-reduced-motion', async () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: true, addEventListener: vi.fn(), removeEventListener: vi.fn() });
    const wrapper = await mountBlock('short text', { autoExpandBlocks: true, showAnimations: true });
    expect(wrapper.html()).not.toContain('motion-breathe');
  });

  it('the breathe effect class IS present while expanded with no reduced-motion preference', async () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() });
    const wrapper = await mountBlock('short text', { autoExpandBlocks: true, showAnimations: true });
    expect(wrapper.html()).toContain('motion-breathe');
  });

  it('collapsed (not expanded) never gets the breathe class either way', async () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() });
    const wrapper = await mountBlock('short text', { autoExpandBlocks: false });
    expect(wrapper.html()).not.toContain('motion-breathe');
  });
});
