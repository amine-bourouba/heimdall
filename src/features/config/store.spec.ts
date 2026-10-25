// M3-11's own AC: "The project picker offers known projects plus a folder chosen via
// pick_config_project; the choice persists in configProjectPath." / "Per-screen layout persists
// in configLayouts."
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { Prefs, ProjectSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useConfigStore } from './store';

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

function makeProject(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return { id: 'p1', name: 'my-project', path: '/Users/me/my-project', pathSource: 'decoded', sessionCount: 3, lastActivity: null, hasMemory: false, ...overrides };
}

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('config store: project restoration (AC1)', () => {
  it('load() fetches known projects and defaults selectedProject to null when no folder is picked', async () => {
    usePrefsStore().prefs = defaultPrefs();
    mockIPC((cmd) => {
      if (cmd === 'list_projects') return { value: [makeProject()], skipped: 0 };
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();

    await store.load();

    expect(store.projects).toEqual([makeProject()]);
    expect(store.selectedProject).toBeNull();
  });

  it('load() defaults selectedProject to pickedFolder when configProjectPath is already set', async () => {
    usePrefsStore().prefs = defaultPrefs({ configProjectPath: '/Users/me/other-project' });
    mockIPC((cmd) => {
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();

    await store.load();

    expect(store.selectedProject).toEqual({ kind: 'pickedFolder' });
    expect(store.pickedFolderName()).toBe('other-project');
  });

  it('a second load() does not re-derive selectedProject from Prefs, preserving an in-session pick', async () => {
    usePrefsStore().prefs = defaultPrefs({ configProjectPath: '/Users/me/other-project' });
    mockIPC((cmd) => {
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();
    await store.load();
    store.selectProject({ kind: 'project', projectId: 'p1' });

    await store.load();

    expect(store.selectedProject).toEqual({ kind: 'project', projectId: 'p1' });
  });
});

describe('config store: browsing for a folder (AC1)', () => {
  it('sets selectedProject to pickedFolder when the native dialog returns "written"', async () => {
    mockIPC((cmd) => {
      if (cmd === 'pick_config_project') return 'written';
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();

    await store.browseForProject();

    expect(store.selectedProject).toEqual({ kind: 'pickedFolder' });
  });

  it('leaves selectedProject unchanged when the dialog is cancelled', async () => {
    mockIPC((cmd) => {
      if (cmd === 'pick_config_project') return 'cancelled';
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();
    store.selectProject({ kind: 'project', projectId: 'p1' });

    await store.browseForProject();

    expect(store.selectedProject).toEqual({ kind: 'project', projectId: 'p1' });
  });
});

describe('config store: per-screen layout (AC2)', () => {
  it('layoutFor() defaults to "grid" for a screen with no entry yet', () => {
    usePrefsStore().prefs = defaultPrefs();
    const store = useConfigStore();

    expect(store.layoutFor('commands')).toBe('grid');
  });

  it('setLayout() spreads the current map so another screen\'s saved layout survives', async () => {
    usePrefsStore().prefs = defaultPrefs({ configLayouts: { skills: 'list' } });
    let patchArgs: unknown = null;
    mockIPC((cmd, args) => {
      if (cmd === 'update_prefs') {
        patchArgs = (args as { patch: { configLayouts: unknown } }).patch.configLayouts;
        const current = usePrefsStore().prefs!;
        usePrefsStore().prefs = { ...current, configLayouts: patchArgs as Record<string, 'grid' | 'list'> };
        return usePrefsStore().prefs;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();

    await store.setLayout('commands', 'list');

    expect(patchArgs).toEqual({ skills: 'list', commands: 'list' });
    expect(store.layoutFor('skills')).toBe('list'); // untouched by the commands write
    expect(store.layoutFor('commands')).toBe('list');
  });

  it('toggleLayout() flips grid to list and back', async () => {
    usePrefsStore().prefs = defaultPrefs();
    mockIPC((cmd, args) => {
      if (cmd === 'update_prefs') {
        const patch = (args as { patch: { configLayouts: Record<string, 'grid' | 'list'> } }).patch;
        usePrefsStore().prefs = { ...usePrefsStore().prefs!, configLayouts: patch.configLayouts };
        return usePrefsStore().prefs;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useConfigStore();

    await store.toggleLayout('hooks');
    expect(store.layoutFor('hooks')).toBe('list');

    await store.toggleLayout('hooks');
    expect(store.layoutFor('hooks')).toBe('grid');
  });
});
