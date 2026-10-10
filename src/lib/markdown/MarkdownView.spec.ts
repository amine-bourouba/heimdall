// @vitest-environment jsdom
//
// Mounts the real MarkdownView, which renders through the real `renderMarkdown()` -> DOMPurify
// pipeline -- see renderMarkdown.spec.ts's own header comment for why that needs a real DOM
// (jsdom) rather than this project's happy-dom default; the same reasoning applies here.
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { renderMarkdown } from './renderMarkdown';
import MarkdownView from './MarkdownView.vue';

// Shiki lazily loads the "javascript" grammar, its WASM engine, and each *theme* bundle
// (`github-light`/`github-dark`) separately on first use -- that first load takes real,
// unpredictable async time no bounded number of `flushPromises()` calls reliably covers
// (confirmed directly, twice: the grammar's own first load needed a dedicated warm-up below, and
// even after that, switching to the dark theme for the very first time in this file still missed
// a single `flushPromises()` -- a trace showed the real highlighted HTML resolving a tick after
// the test's assertion had already run and failed). Warming *both* themes for real, up front,
// avoids guessing at a tick count for the component-level tests below.
beforeAll(async () => {
  await renderMarkdown('```javascript\nwarm\n```', false);
  await renderMarkdown('```javascript\nwarm\n```', true);
});

function defaultPrefs(): Prefs {
  return {
    schemaVersion: 1,
    showMenuBarIcon: true,
    hasCompletedOnboarding: true,
    colorTheme: 'white', // isDark: false, not "adaptive" -- no matchMedia resolution needed.
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
  };
}

let mounted: ReturnType<typeof mount>[] = [];

function mountView(content: string, ipcHandler: (cmd: string, args: unknown) => unknown = () => null) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  mockIPC(ipcHandler);
  const wrapper = mount(MarkdownView, { props: { content } });
  mounted.push(wrapper);
  return wrapper;
}

// jsdom (unlike this project's default happy-dom) has no `matchMedia` at all -- a bare stub is
// enough here since every test below uses a non-"adaptive" colour theme (`white`/`charcoal`),
// so `MarkdownView`'s own OS-dark resolution branch is never actually exercised by its result.
beforeEach(() => {
  document.body.innerHTML = '';
  window.matchMedia = vi.fn().mockReturnValue({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() });
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('MarkdownView: links never navigate the webview directly (ADR 0010 point 2)', () => {
  it('a click on a rendered link is prevented and routed through open_external_url', async () => {
    let calledWith: unknown;
    const wrapper = mountView('[docs](https://example.com/docs)', (cmd, args) => {
      if (cmd === 'open_external_url') calledWith = args;
      return null;
    });
    await flushPromises();

    const link = wrapper.find('a');
    expect(link.exists()).toBe(true);
    // `.trigger()` resolves to `Promise<void>` (confirmed in @vue/test-utils' own types), not
    // the Event -- a real listener is the only way to read `defaultPrevented` back.
    let capturedEvent: Event | undefined;
    link.element.addEventListener('click', (e) => (capturedEvent = e));
    await link.trigger('click');
    expect(capturedEvent?.defaultPrevented).toBe(true);
    expect(calledWith).toEqual({ url: 'https://example.com/docs' });
  });

  it('a click elsewhere in the rendered content (not on a link) does nothing', async () => {
    const wrapper = mountView('plain paragraph, no link');
    await flushPromises();
    await wrapper.find('.markdown-view').trigger('click');
    // No assertion needed beyond "did not throw" -- there is no open_external_url call to miss
    // since mockIPC above would throw on an unexpected command if one were made.
  });
});

describe('MarkdownView: re-renders when content or the resolved theme changes', () => {
  it('changing `content` re-renders with the new markdown', async () => {
    const wrapper = mountView('first');
    await flushPromises();
    expect(wrapper.text()).toContain('first');

    await wrapper.setProps({ content: 'second' });
    await flushPromises();
    expect(wrapper.text()).not.toContain('first');
    expect(wrapper.text()).toContain('second');
  });

  it('switching the active colour theme between a light and a dark one re-highlights code blocks', async () => {
    const wrapper = mountView('```javascript\nconst x = 1;\n```');
    await flushPromises();
    expect(wrapper.html()).toContain('github-light');

    usePrefsStore().prefs = { ...defaultPrefs(), colorTheme: 'charcoal' }; // isDark: true
    await flushPromises();
    expect(wrapper.html()).toContain('github-dark');
  });

  it('only the latest of two rapid content changes ends up rendered (no stale overwrite)', async () => {
    const wrapper = mountView('first');
    await wrapper.setProps({ content: 'second' });
    await wrapper.setProps({ content: 'third' });
    await flushPromises();
    expect(wrapper.text()).toContain('third');
    expect(wrapper.text()).not.toContain('first');
    expect(wrapper.text()).not.toContain('second');
  });
});
