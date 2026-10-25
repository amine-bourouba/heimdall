// M3-16's own AC: "Every preference in parity-matrix section I except showMenuBarIcon and
// launch-at-login is editable here." / "Editor and terminal lists show only options valid for the
// running OS." / "The Menu Bar tab and launch-at-login toggle come later (M3-58, M3-60)."
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { MemorySource, Prefs } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import { usePlatformStore } from '@/stores/platform';
import SettingsPage from './SettingsPage.vue';

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

let mounted: ReturnType<typeof mount>[] = [];

async function mountPage(os: 'macos' | 'windows' | 'linux', ipcHandler?: (cmd: string, args: unknown) => unknown) {
  setActivePinia(createPinia());
  usePlatformStore().os = os;
  let current = defaultPrefs();
  mockIPC((cmd, args) => {
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      current = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return current;
    }
    if (ipcHandler) {
      const result = ipcHandler(cmd, args);
      if (result !== undefined) return result;
    }
    if (cmd === 'list_memory_sources') return [];
    throw new Error(`unexpected command: ${cmd}`);
  }, { shouldMockEvents: true });
  usePrefsStore().prefs = current;
  const router = createRouter({ history: createWebHashHistory(), routes: [{ path: '/', name: 'sessions', component: { render: () => null } }] });
  await router.push('/');
  const wrapper = mount(SettingsPage, { global: { plugins: [router] }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return { wrapper, getPrefs: () => current };
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('SettingsPage: General tab', () => {
  it('editing Default Editor, Terminal, Open Terminal Automatically and Claude Code Path all write real prefs', async () => {
    const { wrapper, getPrefs } = await mountPage('windows');
    const selects = wrapper.findAll('select');

    await selects[0].setValue('cursor');
    expect(getPrefs().textEditor).toBe('cursor');

    await selects[1].setValue('cmd');
    expect(getPrefs().preferredTerminal).toBe('cmd');

    await wrapper.find('input[type="checkbox"]').setValue(true);
    expect(getPrefs().openTerminalOnBash).toBe(true);

    await wrapper.find('input[type="text"]').setValue('C:\\claude.exe');
    expect(getPrefs().claudeCodePath).toBe('C:\\claude.exe');
  });
});

describe('SettingsPage: editor/terminal lists are OS-filtered (AC)', () => {
  it('offers Xcode only on macOS', async () => {
    const macos = await mountPage('macos');
    expect(macos.wrapper.find('option[value="xcode"]').exists()).toBe(true);

    const windows = await mountPage('windows');
    expect(windows.wrapper.find('option[value="xcode"]').exists()).toBe(false);
  });

  it.each([
    ['macos', ['terminal', 'iterm2', 'warp', 'ghostty', 'kitty', 'alacritty']],
    ['windows', ['windowsTerminal', 'powershell', 'cmd']],
    ['linux', ['gnomeTerminal', 'konsole', 'kitty', 'alacritty', 'system']],
  ] as const)('terminal options on %s match PreferredTerminal::options exactly', async (os, expected) => {
    const { wrapper } = await mountPage(os);
    const options = wrapper.findAll('select')[1].findAll('option').map((o) => o.attributes('value'));
    expect(options).toEqual([...expected]);
  });
});

describe('SettingsPage: Appearance tab', () => {
  it('selecting a theme writes colorTheme', async () => {
    const { wrapper, getPrefs } = await mountPage('windows');
    await wrapper.find('button[title="Appearance"]').trigger('click');

    await wrapper.find('button[title="Mocha"]').trigger('click');
    await flushPromises();

    expect(getPrefs().colorTheme).toBe('mocha');
  });

  it('toggling animations and nudging font scale both write real prefs', async () => {
    const { wrapper, getPrefs } = await mountPage('windows');
    await wrapper.find('button[title="Appearance"]').trigger('click');

    await wrapper.find('input[type="checkbox"]').setValue(false);
    expect(getPrefs().showAnimations).toBe(false);

    await wrapper.find('button[title="Increase font size"]').trigger('click');
    await flushPromises();
    expect(getPrefs().fontScale).toBe(1.05);
  });
});

describe('SettingsPage: Viewer tab', () => {
  it('each of the three toggles writes its own prefs field', async () => {
    const { wrapper, getPrefs } = await mountPage('windows');
    await wrapper.find('button[title="Viewer"]').trigger('click');

    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    await checkboxes[0].setValue(false);
    await checkboxes[1].setValue(false);
    await checkboxes[2].setValue(false);

    expect(getPrefs().wrapCodeLines).toBe(false);
    expect(getPrefs().autoExpandBlocks).toBe(false);
    expect(getPrefs().parseMarkdownInResults).toBe(false);
  });
});

describe('SettingsPage: Memory tab (reuses memoryStore.addFolder/removeFolder, M3-04)', () => {
  function makeFolder(overrides: Partial<MemorySource> = {}): MemorySource {
    return { id: 'folder:0', label: '/Users/me/notes', kind: 'folder', fileCount: 3, ...overrides };
  }

  it('lists only folder-kind sources, not project memory sources', async () => {
    const { wrapper } = await mountPage('windows', (cmd) => {
      if (cmd === 'list_memory_sources') {
        return [makeFolder(), { id: 'project:abc', label: 'my-project', kind: 'project', fileCount: 1 }];
      }
      if (cmd === 'list_memory_files') return [];
    });
    await wrapper.find('button[title="Memory"]').trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('/Users/me/notes');
    expect(wrapper.text()).not.toContain('my-project');
  });

  it('Add Folder… calls add_memory_folder and Remove calls remove_memory_folder', async () => {
    let addCalled = false;
    let removeArgs: unknown = null;
    const { wrapper } = await mountPage('windows', (cmd, args) => {
      if (cmd === 'list_memory_sources') return [makeFolder()];
      if (cmd === 'list_memory_files') return [];
      if (cmd === 'add_memory_folder') {
        addCalled = true;
        return 'written';
      }
      if (cmd === 'remove_memory_folder') {
        removeArgs = args;
        return 'written';
      }
    });
    await wrapper.find('button[title="Memory"]').trigger('click');
    await flushPromises();

    await wrapper.find('[title="Remove folder"]').trigger('click');
    await flushPromises();
    expect(removeArgs).toEqual({ sourceId: 'folder:0' });

    const addButton = wrapper.findAll('button').find((b) => b.text().includes('Add Folder'));
    await addButton!.trigger('click');
    await flushPromises();
    expect(addCalled).toBe(true);
  });
});

describe('SettingsPage: no Menu Bar tab or launch-at-login toggle (AC3)', () => {
  it('renders exactly four tabs, none of them Menu Bar', async () => {
    const { wrapper } = await mountPage('windows');
    const tabs = wrapper.findAll('[role="tab"]').map((t) => t.text());
    expect(tabs).toEqual(['General', 'Appearance', 'Viewer', 'Memory']);
  });

  it('renders no showMenuBarIcon or launch-at-login control anywhere', async () => {
    const { wrapper } = await mountPage('windows');
    for (const tab of ['General', 'Appearance', 'Viewer', 'Memory']) {
      await wrapper.find(`button[title="${tab}"]`).trigger('click');
      expect(wrapper.text().toLowerCase()).not.toContain('menu bar');
      expect(wrapper.text().toLowerCase()).not.toContain('launch at login');
    }
  });
});

describe('SettingsPage: Done', () => {
  it('Done navigates back', async () => {
    setActivePinia(createPinia());
    usePlatformStore().os = 'windows';
    mockIPC((cmd) => {
      if (cmd === 'list_memory_sources') return [];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    usePrefsStore().prefs = defaultPrefs();
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [
        { path: '/', name: 'sessions', component: { render: () => null } },
        { path: '/settings', name: 'settings', component: SettingsPage },
      ],
    });
    await router.push('/');
    await router.push('/settings');
    const wrapper = mount(SettingsPage, { global: { plugins: [router] }, attachTo: document.body });
    mounted.push(wrapper);
    await flushPromises();

    await wrapper.find('button[title="Done"]').trigger('click');
    await flushPromises();

    expect(router.currentRoute.value.name).toBe('sessions');
  });
});
