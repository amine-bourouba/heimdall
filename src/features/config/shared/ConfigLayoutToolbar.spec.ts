import { defineComponent, h, ref } from 'vue';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { mockIPC } from '@tauri-apps/api/mocks';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Prefs } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';
import ConfigLayoutToolbar from './ConfigLayoutToolbar.vue';

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

beforeEach(() => {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  mockIPC((cmd, args) => {
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: { configLayouts: unknown } }).patch;
      usePrefsStore().prefs = { ...usePrefsStore().prefs!, configLayouts: patch.configLayouts as Record<string, 'grid' | 'list'> };
      return usePrefsStore().prefs;
    }
    if (cmd === 'list_projects') return { value: [], skipped: 0 };
    throw new Error(`unexpected command: ${cmd}`);
  });
});

// A real two-way-bound host, not wrapper.emitted(): FilterField.spec.ts's own header comment
// documents why @vue/test-utils' emit tracking never fires in this project (it depends on
// Vue.setDevtoolsHook, present only in the esm-browser build, not the bundler/cjs build Vitest
// actually resolves here) -- confirmed again directly while writing this file's own first draft.
function mountWithHost(filterQuery: string) {
  const Host = defineComponent({
    setup() {
      const query = ref(filterQuery);
      return () =>
        h(ConfigLayoutToolbar, {
          screenId: 'commands',
          filterQuery: query.value,
          'onUpdate:filterQuery': (v: string) => (query.value = v),
          filterPlaceholder: 'Find in Commands…',
        });
    },
  });
  return mount(Host, { attachTo: document.body });
}

describe('ConfigLayoutToolbar: filter field (reuses FilterField.vue, M2-07)', () => {
  it('two-way binds filterQuery through FilterField', async () => {
    const wrapper = mountWithHost('');
    const input = wrapper.find('input[placeholder="Find in Commands…"]');

    await input.setValue('deploy');

    expect((input.element as HTMLInputElement).value).toBe('deploy');
  });
});

describe('ConfigLayoutToolbar: project picker visibility (config-screens.md §1.2)', () => {
  it('shows the project picker only when showProjectPicker is true', () => {
    const shown = mount(ConfigLayoutToolbar, { props: { screenId: 'commands', filterQuery: '', filterPlaceholder: '', showProjectPicker: true } });
    expect(shown.text()).toContain('No project selected');

    const hidden = mount(ConfigLayoutToolbar, { props: { screenId: 'models', filterQuery: '', filterPlaceholder: '', showProjectPicker: false } });
    expect(hidden.text()).not.toContain('No project selected');
  });
});

describe('ConfigLayoutToolbar: add button visibility (config-screens.md §1.2)', () => {
  it('calls the onAdd listener only when the button is shown', async () => {
    const onAdd = vi.fn();
    const wrapper = mount(ConfigLayoutToolbar, { props: { screenId: 'commands', filterQuery: '', filterPlaceholder: '', showAddButton: true, onAdd } });
    await wrapper.find('button[title="Add"]').trigger('click');
    expect(onAdd).toHaveBeenCalledTimes(1);
  });

  it('renders no Add button when showAddButton is false', () => {
    const wrapper = mount(ConfigLayoutToolbar, { props: { screenId: 'models', filterQuery: '', filterPlaceholder: '', showAddButton: false } });
    expect(wrapper.find('button[title="Add"]').exists()).toBe(false);
  });
});

describe('ConfigLayoutToolbar: layout toggle (AC2)', () => {
  it('toggling flips this screen\'s own persisted layout via the shared store', async () => {
    const wrapper = mount(ConfigLayoutToolbar, { props: { screenId: 'commands', filterQuery: '', filterPlaceholder: '' } });

    await wrapper.find('button[title="Switch to list"]').trigger('click');

    expect(usePrefsStore().prefs?.configLayouts).toEqual({ commands: 'list' });
  });
});
