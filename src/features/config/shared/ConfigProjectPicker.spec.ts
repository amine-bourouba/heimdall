// AC1: "The project picker offers known projects plus a folder chosen via pick_config_project;
// the choice persists in configProjectPath."
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { Prefs, ProjectSummary } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { useConfigStore } from '../store';
import ConfigProjectPicker from './ConfigProjectPicker.vue';

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

let mounted: ReturnType<typeof mount>[] = [];

async function mountPicker(prefs: Prefs, ipcHandler: (cmd: string, args: unknown) => unknown) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = prefs;
  mockIPC((cmd, args) => {
    const result = ipcHandler(cmd, args);
    if (result !== undefined) return result;
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(ConfigProjectPicker, { attachTo: document.body });
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

describe('ConfigProjectPicker: known projects', () => {
  it('lists every known project as an option and shows "No project selected" by default', async () => {
    const wrapper = await mountPicker(defaultPrefs(), (cmd) => {
      if (cmd === 'list_projects') return { value: [makeProject()], skipped: 0 };
    });
    expect(wrapper.text()).toContain('No project selected');
    expect(wrapper.find('option[value="p1"]').text()).toBe('my-project');
  });

  it('selecting a known project updates the current label and the store\'s selectedProject', async () => {
    const wrapper = await mountPicker(defaultPrefs(), (cmd) => {
      if (cmd === 'list_projects') return { value: [makeProject()], skipped: 0 };
    });

    await wrapper.find('select[aria-label="Jump to a known project"]').setValue('p1');

    expect(useConfigStore().selectedProject).toEqual({ kind: 'project', projectId: 'p1' });
    expect(wrapper.text()).toContain('my-project');
  });
});

describe('ConfigProjectPicker: browsing for a folder (AC1)', () => {
  it('Browse… calls pick_config_project and shows the picked folder\'s name once written', async () => {
    const wrapper = await mountPicker(defaultPrefs(), (cmd) => {
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
      if (cmd === 'pick_config_project') {
        usePrefsStore().prefs = { ...usePrefsStore().prefs!, configProjectPath: '/Users/me/picked' };
        return 'written';
      }
    });

    await wrapper.find('button[title="Browse for a folder"]').trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('picked');
  });

  it('shows the already-persisted picked folder on first load, before any click', async () => {
    const wrapper = await mountPicker(defaultPrefs({ configProjectPath: '/Users/me/already-picked' }), (cmd) => {
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });
    expect(wrapper.text()).toContain('already-picked');
  });
});

describe('ConfigProjectPicker: clear', () => {
  it('shows an "x" only when a project is active, and clears the in-memory selection on click', async () => {
    const wrapper = await mountPicker(defaultPrefs({ configProjectPath: '/Users/me/picked' }), (cmd) => {
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });
    expect(wrapper.find('button[title="Clear project"]').exists()).toBe(true);

    await wrapper.find('button[title="Clear project"]').trigger('click');

    expect(wrapper.text()).toContain('No project selected');
    expect(wrapper.find('button[title="Clear project"]').exists()).toBe(false);
    // The underlying Prefs.configProjectPath is untouched -- there is no command to clear it.
    expect(usePrefsStore().prefs?.configProjectPath).toBe('/Users/me/picked');
  });
});
