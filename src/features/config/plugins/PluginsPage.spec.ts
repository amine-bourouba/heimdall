// M3-50 AC: "Each action calls its contract command and shows a toast on success or a clear error
// on failure." Plugins: enable/disable (the Enabled/Disabled pill) and remove (the trash button).
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { PluginInfo } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import PluginsPage from './PluginsPage.vue';

function plugin(overrides: Partial<PluginInfo> = {}): PluginInfo {
  return { key: 'formatter@acme', name: 'formatter', author: 'acme', version: '1.2.0', installedAt: null, enabled: true, ...overrides };
}

const busy = () => {
  throw { kind: 'busy', message: 'settings.json is in use by another program', path: null };
};

interface Seen {
  lists: number;
  toggled: unknown[];
  removed: unknown[];
}

let mounted: ReturnType<typeof mount>[] = [];

/** `lists` is what each successive list_plugins call returns (the last one repeats). */
async function mountPage(lists: PluginInfo[][], overrides: Partial<Record<string, (args: unknown) => unknown>> = {}) {
  setActivePinia(createPinia());
  const seen: Seen = { lists: 0, toggled: [], removed: [] };
  mockIPC((cmd, args) => {
    if (overrides[cmd]) return overrides[cmd]!(args);
    switch (cmd) {
      case 'list_plugins':
        return lists[Math.min(seen.lists++, lists.length - 1)];
      case 'set_plugin_enabled':
        seen.toggled.push(args);
        return 'written';
      case 'remove_plugin':
        seen.removed.push(args);
        return 'written';
      default:
        throw new Error(`unexpected command: ${cmd}`);
    }
  });
  const wrapper = mount(PluginsPage, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return { wrapper, seen, toasts: useToastsStore() };
}

const toggleButton = (wrapper: ReturnType<typeof mount>) => wrapper.find('button[role="switch"]');
const removeButton = (wrapper: ReturnType<typeof mount>) => wrapper.find('button[title="Remove from enabled plugins"]');

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.unstubAllGlobals();
});

describe('PluginsPage: the Enabled/Disabled pill is a switch', () => {
  it('names the plugin and its state, and reports the state to assistive technology', async () => {
    const { wrapper } = await mountPage([[plugin({ enabled: true }), plugin({ key: 'linter@acme', name: 'linter', enabled: false })]]);

    const switches = wrapper.findAll('button[role="switch"]');
    expect(switches.map((s) => s.attributes('aria-label'))).toEqual(['formatter: Enabled', 'linter: Disabled']);
    expect(switches.map((s) => s.attributes('aria-checked'))).toEqual(['true', 'false']);
  });

  it('enables a disabled plugin by its key, says so, and shows the new state without the skeleton', async () => {
    const { wrapper, seen, toasts } = await mountPage([[plugin({ enabled: false })], [plugin({ enabled: true })]]);
    expect(toggleButton(wrapper).attributes('aria-checked')).toBe('false');

    await toggleButton(wrapper).trigger('click');
    await flushPromises();

    expect(seen.toggled).toEqual([{ key: 'formatter@acme', enabled: true }]);
    expect(toasts.current).toMatchObject({ message: 'formatter enabled', style: 'success' });
    expect(toggleButton(wrapper).attributes('aria-checked')).toBe('true');
  });

  it('disables an enabled plugin and reports it as plain information', async () => {
    const { wrapper, seen, toasts } = await mountPage([[plugin({ enabled: true })], [plugin({ enabled: false })]]);

    await toggleButton(wrapper).trigger('click');
    await flushPromises();

    expect(seen.toggled).toEqual([{ key: 'formatter@acme', enabled: false }]);
    expect(toasts.current).toMatchObject({ message: 'formatter disabled', style: 'info' });
    expect(toggleButton(wrapper).attributes('aria-checked')).toBe('false');
  });

  it('shows the cause when the write fails, leaves the state alone, and can be retried', async () => {
    let fail = true;
    const { wrapper, seen, toasts } = await mountPage([[plugin({ enabled: true })]], {
      set_plugin_enabled: () => (fail ? busy() : 'written'),
    });

    await toggleButton(wrapper).trigger('click');
    await flushPromises();

    expect(toasts.current).toMatchObject({ message: 'settings.json is in use by another program', style: 'error' });
    expect(toggleButton(wrapper).attributes('aria-checked')).toBe('true');
    expect(seen.lists).toBe(1);

    fail = false;
    await toggleButton(wrapper).trigger('click');
    await flushPromises();
    expect(seen.lists).toBe(2);
  });

  it('is disabled while a write is in flight, so a second click cannot queue another', async () => {
    let release: (value: unknown) => void = () => {};
    let calls = 0;
    const { wrapper } = await mountPage([[plugin({ enabled: true })]], {
      set_plugin_enabled: () => {
        calls++;
        return new Promise((resolve) => (release = resolve));
      },
    });

    await toggleButton(wrapper).trigger('click');
    expect(toggleButton(wrapper).attributes('disabled')).toBeDefined();
    expect(removeButton(wrapper).attributes('disabled')).toBeDefined();
    await toggleButton(wrapper).trigger('click');

    release('written');
    await flushPromises();
    expect(calls).toBe(1);
    expect(toggleButton(wrapper).attributes('disabled')).toBeUndefined();
  });
});

describe('PluginsPage: remove', () => {
  it('asks first, saying the plugin stays installed, and sends nothing if declined', async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);
    const { wrapper, seen } = await mountPage([[plugin()]]);

    await removeButton(wrapper).trigger('click');
    await flushPromises();

    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(confirmSpy.mock.calls[0][0]).toContain('"formatter"');
    expect(confirmSpy.mock.calls[0][0]).toContain('stays installed');
    expect(seen.removed).toEqual([]);
  });

  it('removes the plugin by its key once confirmed, says what that did, and reloads the list', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    // The plugin is still installed, so it is still listed afterwards, as Disabled.
    const { wrapper, seen, toasts } = await mountPage([[plugin({ enabled: true })], [plugin({ enabled: false })]]);

    await removeButton(wrapper).trigger('click');
    await flushPromises();

    expect(seen.removed).toEqual([{ key: 'formatter@acme' }]);
    expect(toasts.current).toMatchObject({ message: 'Removed formatter from enabled plugins', style: 'info' });
    expect(seen.lists).toBe(2);
    expect(toggleButton(wrapper).attributes('aria-label')).toBe('formatter: Disabled');
  });

  it('shows the cause when the write fails', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    const { wrapper, toasts } = await mountPage([[plugin()]], { remove_plugin: busy });

    await removeButton(wrapper).trigger('click');
    await flushPromises();

    expect(toasts.current).toMatchObject({ message: 'settings.json is in use by another program', style: 'error' });
  });

  it('has an accessible name that says which plugin', async () => {
    const { wrapper } = await mountPage([[plugin({ name: 'linter', key: 'linter@acme' })]]);
    expect(removeButton(wrapper).attributes('aria-label')).toBe('Remove linter from enabled plugins');
  });
});

describe('PluginsPage: loading', () => {
  it('shows the cause and the empty state, not a skeleton forever, when the list cannot be read', async () => {
    const { wrapper, toasts } = await mountPage([[]], { list_plugins: busy });

    expect(toasts.current).toMatchObject({ message: 'settings.json is in use by another program', style: 'error' });
    expect(wrapper.text()).toContain('No plugins found');
  });

  it('still filters by name', async () => {
    const { wrapper } = await mountPage([[plugin(), plugin({ key: 'linter@acme', name: 'linter' })]]);

    await wrapper.find('input[aria-label="Find in Plugins…"]').setValue('lint');
    await flushPromises();

    expect(wrapper.findAll('button[role="switch"]').map((s) => s.attributes('aria-label'))).toEqual(['linter: Enabled']);
  });
});
