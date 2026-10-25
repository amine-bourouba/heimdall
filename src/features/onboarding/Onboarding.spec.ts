// M3-15's own AC: "<- and -> move between pages; completion sets hasCompletedOnboarding." /
// "Detection uses detect_claude_cli." Ported from OnboardingView.swift. Every query goes through
// `document.body`, not `wrapper.find()` -- this component's root is a `<Teleport to="body">`
// (SearchOverlay.spec.ts's own documented reasoning: Vue Test Utils does not search teleported
// content from the mounting wrapper).
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { CliStatus, Prefs } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { usePlatformStore } from '@/stores/platform';
import Onboarding from './Onboarding.vue';

function defaultPrefs(overrides: Partial<Prefs> = {}): Prefs {
  return {
    schemaVersion: 1,
    showMenuBarIcon: true,
    hasCompletedOnboarding: false,
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

function body() {
  return new DOMWrapper(document.body);
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountOnboarding(cliStatus: CliStatus, ipcHandler?: (cmd: string, args: unknown) => unknown) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  usePlatformStore().os = 'macos';
  mockIPC((cmd, args) => {
    if (cmd === 'detect_claude_cli') return cliStatus;
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      const current = usePrefsStore().prefs!;
      usePrefsStore().prefs = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return usePrefsStore().prefs;
    }
    if (ipcHandler) return ipcHandler(cmd, args);
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(Onboarding, { attachTo: document.body });
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
});

describe('Onboarding: page navigation (AC)', () => {
  it('Continue and -> both move forward; <- moves back; Back is disabled on page 0', async () => {
    await mountOnboarding({ cliPath: '/usr/local/bin/claude', sessionsExist: true });
    expect(body().text()).toContain('Welcome to Heimdall');
    expect(body().find('button[title="Go back"]').attributes('disabled')).toBeDefined();

    await body().find('button[title="Go to the next step"]').trigger('click');
    expect(body().text()).toContain('Claude Code Detected');

    await body().trigger('keydown', { key: 'ArrowRight' });
    expect(body().text()).toContain('Key Features');
    expect(body().find('button[title="Go to the next step"]').exists()).toBe(false);

    await body().trigger('keydown', { key: 'ArrowLeft' });
    expect(body().text()).toContain('Claude Code Detected');
  });

  it('ArrowRight/ArrowLeft at the boundary pages are no-ops', async () => {
    await mountOnboarding({ cliPath: null, sessionsExist: false });

    await body().trigger('keydown', { key: 'ArrowLeft' });
    expect(body().text()).toContain('Welcome to Heimdall');
  });
});

describe('Onboarding: completion sets hasCompletedOnboarding (AC)', () => {
  it('Get Started on the last page persists hasCompletedOnboarding: true', async () => {
    await mountOnboarding({ cliPath: '/usr/local/bin/claude', sessionsExist: false });
    await body().find('button[title="Go to the next step"]').trigger('click');
    await body().find('button[title="Go to the next step"]').trigger('click');

    await body().find('button[title="Get started with Heimdall"]').trigger('click');
    await flushPromises();

    expect(usePrefsStore().prefs?.hasCompletedOnboarding).toBe(true);
  });
});

describe('Onboarding: CLI detection (AC)', () => {
  it('calls detect_claude_cli and shows the detected state with sessions found', async () => {
    await mountOnboarding({ cliPath: '/usr/local/bin/claude', sessionsExist: true });
    await body().find('button[title="Go to the next step"]').trigger('click');

    expect(body().text()).toContain('Claude Code Detected');
    expect(body().text()).toContain('Sessions found');
  });

  it('shows the not-found state with the install command when no CLI is detected', async () => {
    await mountOnboarding({ cliPath: null, sessionsExist: false });
    await body().find('button[title="Go to the next step"]').trigger('click');

    expect(body().text()).toContain('Claude Code Not Found');
    expect(body().text()).toContain('npm install -g @anthropic-ai/claude-code');
  });

  it('shows "No sessions yet" when the CLI is found but no sessions exist', async () => {
    await mountOnboarding({ cliPath: '/usr/local/bin/claude', sessionsExist: false });
    await body().find('button[title="Go to the next step"]').trigger('click');

    expect(body().text()).toContain('No sessions yet');
  });
});

describe('Onboarding: per-OS install hint', () => {
  it('shows a Windows-specific hint when not found on Windows', async () => {
    setActivePinia(createPinia());
    usePrefsStore().prefs = defaultPrefs();
    usePlatformStore().os = 'windows';
    mockIPC((cmd) => {
      if (cmd === 'detect_claude_cli') return { cliPath: null, sessionsExist: false } satisfies CliStatus;
      throw new Error(`unexpected command: ${cmd}`);
    });
    const wrapper = mount(Onboarding, { attachTo: document.body });
    mounted.push(wrapper);
    await flushPromises();
    await body().find('button[title="Go to the next step"]').trigger('click');

    expect(body().text()).toContain('PowerShell or Command Prompt');
  });
});
