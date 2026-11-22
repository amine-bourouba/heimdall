// AC3: "The detail toolbar copies name and content; editor and reveal buttons come later
// (M3-55)." Close (M3-12's own necessary ripple, ConfigDetailToolbar.swift:136-148) is covered
// below too, and so are the editor and reveal buttons M3-55 added (last describe block).
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { vi, beforeEach, describe, expect, it, afterEach } from 'vitest';
import type { ConfigRef, Prefs, PreferredEditor, TargetOs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { usePrefsStore } from '@/stores/prefs';
import { useToastsStore } from '@/stores/toasts';
import ConfigDetailToolbar from './ConfigDetailToolbar.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

beforeEach(() => {
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore() unconditionally.
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ConfigDetailToolbar: copy name and content (AC3)', () => {
  it('copies the name and shows a checkmark for 1.5s', async () => {
    vi.useFakeTimers();
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = mount(ConfigDetailToolbar, { props: { name: '/deploy', content: 'body text', formatted: true } });

    await wrapper.find('button[title="Copy Name"]').trigger('click');
    expect(writeText).toHaveBeenCalledWith('/deploy');
    expect(wrapper.html()).toContain('lucide-check');

    await vi.advanceTimersByTimeAsync(1500);
    expect(wrapper.html()).toContain('lucide-files');
  });

  it('copies the content', async () => {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = mount(ConfigDetailToolbar, { props: { name: '/deploy', content: 'body text', formatted: true } });

    await wrapper.find('button[title="Copy Content"]').trigger('click');
    expect(writeText).toHaveBeenCalledWith('body text');
  });
});

describe('ConfigDetailToolbar: format toggle (read-only, no IPC)', () => {
  // Not wrapper.emitted(): see ConfigLayoutToolbar.spec.ts's own header comment -- @vue/test-utils'
  // emit tracking never fires in this project. A real onUpdate:formatted listener is unaffected.
  it('calls onUpdate:formatted with the flipped value', async () => {
    const onUpdateFormatted = vi.fn();
    const wrapper = mount(ConfigDetailToolbar, { props: { name: 'n', content: 'c', formatted: true, 'onUpdate:formatted': onUpdateFormatted } });

    await wrapper.find('button[title="Show raw content"]').trigger('click');

    expect(onUpdateFormatted).toHaveBeenCalledWith(false);
  });
});

describe('ConfigDetailToolbar: delete (M3-50, opt-in)', () => {
  it('has no Delete button unless the caller says the item can be deleted', () => {
    const wrapper = mount(ConfigDetailToolbar, { props: { name: 'n', content: 'c', formatted: true } });
    expect(wrapper.find('button[title="Delete"]').exists()).toBe(false);
  });

  it('shows a labelled Delete button when deletable, and calls onDelete when clicked', async () => {
    const onDelete = vi.fn();
    const wrapper = mount(ConfigDetailToolbar, { props: { name: 'n', content: 'c', formatted: true, deletable: true, onDelete } });
    const button = wrapper.find('button[title="Delete"]');

    expect(button.attributes('aria-label')).toBe('Delete');
    await button.trigger('click');

    expect(onDelete).toHaveBeenCalledTimes(1);
  });
});

describe('ConfigDetailToolbar: close', () => {
  it('calls onClose when clicked', async () => {
    const onClose = vi.fn();
    const wrapper = mount(ConfigDetailToolbar, { props: { name: 'n', content: 'c', formatted: true, onClose } });

    await wrapper.find('button[title="Close"]').trigger('click');

    expect(onClose).toHaveBeenCalledTimes(1);
  });
});

// M3-55. The commands take the item's reference, never a path (ADR 0010), so the assertions are on the
// exact target. AC2: "Buttons are hidden for editors or terminals not available on the running OS."
describe('ConfigDetailToolbar: open in the editor and show in the file manager (M3-55)', () => {
  const configRef: ConfigRef = { kind: 'command', slug: 'deploy', project: null };
  let launched: [string, unknown][];

  function prefs(textEditor: PreferredEditor): Prefs {
    return {
      schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1, sessionLayout: 'grid',
      configProjectPath: null, configLayouts: {}, showAgentSessions: false, textEditor, preferredTerminal: 'terminal',
      openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true, autoExpandBlocks: true,
      parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [],
    };
  }

  function mountToolbar(options: { ref?: ConfigRef | null; os?: TargetOs | null; editor?: PreferredEditor; fail?: string } = {}) {
    usePlatformStore().os = options.os === undefined ? 'macos' : options.os;
    usePrefsStore().prefs = prefs(options.editor ?? 'vscode');
    launched = [];
    mockIPC((cmd, args) => {
      if (cmd === 'open_in_editor' || cmd === 'reveal_in_file_manager') {
        launched.push([cmd, args]);
        if (options.fail) throw { kind: 'notFound', message: options.fail, path: null };
        return null;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    const props = { name: '/deploy', content: 'c', formatted: true, ...(options.ref === null ? {} : { configRef: options.ref ?? configRef }) };
    return mount(ConfigDetailToolbar, { props });
  }

  it('opens this item in the preferred editor by its reference', async () => {
    const wrapper = mountToolbar();
    await wrapper.find('button[title="Open in VS Code"]').trigger('click');
    await flushPromises();
    expect(launched).toEqual([['open_in_editor', { target: { kind: 'configItem', configRef } }]]);
  });

  it('shows this item in the file manager by its reference', async () => {
    const wrapper = mountToolbar();
    await wrapper.find('button[title="Show in Finder"]').trigger('click');
    await flushPromises();
    expect(launched).toEqual([['reveal_in_file_manager', { target: { kind: 'configItem', configRef } }]]);
  });

  it('passes a project item\'s project along, so the right file is the one opened', async () => {
    const projectRef: ConfigRef = { kind: 'skill', slug: 'lint', project: { kind: 'project', projectId: 'p-42' } };
    const wrapper = mountToolbar({ ref: projectRef });
    await wrapper.find('button[title="Open in VS Code"]').trigger('click');
    await flushPromises();
    expect(launched).toEqual([['open_in_editor', { target: { kind: 'configItem', configRef: projectRef } }]]);
  });

  it('offers neither button when the caller does not say which item this is', () => {
    const wrapper = mountToolbar({ ref: null });
    expect(wrapper.find('button[title^="Open in"]').exists()).toBe(false);
    expect(wrapper.find('button[title^="Show in"]').exists()).toBe(false);
  });

  it('hides the editor button when the saved editor does not exist on this OS (AC2), but keeps the file manager', () => {
    for (const os of ['windows', 'linux'] as const) {
      const wrapper = mountToolbar({ os, editor: 'xcode' });
      expect(wrapper.find('button[title^="Open in"]').exists()).toBe(false);
      expect(wrapper.find('button[title^="Show in"]').exists()).toBe(true);
    }
    expect(mountToolbar({ os: 'macos', editor: 'xcode' }).find('button[title="Open in Xcode"]').exists()).toBe(true);
  });

  it.each<[TargetOs | null, string]>([
    ['macos', 'Finder'],
    ['windows', 'File Explorer'],
    ['linux', 'file manager'],
    [null, 'file manager'],
  ])('names the file manager for the OS (%s: %s)', (os, name) => {
    const button = mountToolbar({ os }).find(`button[title="Show in ${name}"]`);
    expect(button.exists()).toBe(true);
    expect(button.attributes('aria-label')).toBe(`Show in ${name}`);
  });

  it('does not close or delete when either is clicked', async () => {
    const onClose = vi.fn();
    const onDelete = vi.fn();
    const wrapper = mountToolbar();
    await wrapper.setProps({ onClose, onDelete });
    await wrapper.find('button[title="Open in VS Code"]').trigger('click');
    await wrapper.find('button[title="Show in Finder"]').trigger('click');
    await flushPromises();
    expect(onClose).not.toHaveBeenCalled();
    expect(onDelete).not.toHaveBeenCalled();
    expect(launched.map(([cmd]) => cmd)).toEqual(['open_in_editor', 'reveal_in_file_manager']);
  });

  it('shows why when the editor cannot be opened', async () => {
    const wrapper = mountToolbar({ fail: 'the `code` command is not on your PATH' });
    await wrapper.find('button[title="Open in VS Code"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'the `code` command is not on your PATH', style: 'error' });
  });

  it('shows why when the file manager cannot be opened', async () => {
    const wrapper = mountToolbar({ fail: 'the file is gone' });
    await wrapper.find('button[title="Show in Finder"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'the file is gone', style: 'error' });
  });
});
