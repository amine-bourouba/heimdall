import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { Prefs } from '@/ipc/bindings';
import { usePrefsStore } from './prefs';

// ADR 0007's own defaults table, not a hand-picked subset -- every field update() must be able
// to leave untouched when patching just one of the others.
function defaultPrefs(): Prefs {
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
  };
}

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('usePrefsStore', () => {
  it('load() populates prefs and the getters read from it', async () => {
    mockIPC((cmd) => {
      if (cmd === 'get_prefs') return defaultPrefs();
      throw new Error(`unexpected command: ${cmd}`);
    });

    const store = usePrefsStore();
    expect(store.prefs).toBeNull();
    await store.load();

    expect(store.prefs).toEqual(defaultPrefs());
    expect(store.fontScale).toBe(1.0);
    expect(store.colorTheme).toBe('default');
    expect(store.showAnimations).toBe(true);
  });

  it('getters fall back to the ADR 0007 defaults before load() resolves', () => {
    const store = usePrefsStore();
    expect(store.fontScale).toBe(1.0);
    expect(store.colorTheme).toBe('default');
    expect(store.showAnimations).toBe(true);
  });

  it('update() sends a full patch (every other field null) and takes the real response, not an optimistic guess', async () => {
    let sentPatch: unknown;
    mockIPC((cmd, args) => {
      if (cmd === 'update_prefs') {
        sentPatch = (args as { patch: unknown }).patch;
        return { ...defaultPrefs(), fontScale: 1.1 };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const store = usePrefsStore();
    await store.update({ fontScale: 1.1 });

    expect(sentPatch).toMatchObject({ fontScale: 1.1, colorTheme: null, showAnimations: null });
    expect(store.fontScale).toBe(1.1);
  });

  it('update() rejects with the real IpcError when the backend validation fails', async () => {
    mockIPC(() => {
      throw { kind: 'invalidInput', message: 'fontScale must be between 0.75 and 1.5', path: null };
    });

    const store = usePrefsStore();
    await expect(store.update({ fontScale: 99 })).rejects.toMatchObject({ kind: 'invalidInput' });
  });
});
