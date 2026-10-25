import { defineComponent, h } from 'vue';
import { flushPromises, mount } from '@vue/test-utils';
import type { VueWrapper } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { AppAction, Prefs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { usePrefsStore } from '@/stores/prefs';
import { useThemeStore } from '@/stores/theme';
import { useAppActions } from './useAppActions';
import { useShortcuts } from './useShortcuts';

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

function fireKeydown(init: KeyboardEventInit) {
  document.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }));
}

// useShortcuts attaches a document-level keydown listener, and useAppActions' registry is a
// module-level singleton -- both leak across tests without explicit cleanup (the same real leak
// confirmed in useKeyboardNav.spec.ts: a prior test's still-attached listener or still-registered
// handler double-fires into a later test). Tracked and torn down in afterEach below.
let mounted: VueWrapper[] = [];
let unregisterFns: (() => void)[] = [];

async function mountWith(os: 'macos' | 'windows' | 'linux') {
  setActivePinia(createPinia());
  usePlatformStore().os = os;
  // 'settings' is a real route name (matching routes.ts) because useShortcuts now registers its
  // own real handler for it (`router.push({name: 'settings'})`, M3-16) alongside this test's own
  // fake tracking handler -- both fire on every dispatch (useAppActions' registry is a Set, not a
  // single slot), so the router needs somewhere real to navigate to.
  const router = createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/', component: { render: () => null } },
      { path: '/settings', name: 'settings', component: { render: () => null } },
    ],
  });
  await router.push('/');

  const dispatched: AppAction[] = [];
  const { on } = useAppActions();
  for (const action of ['search', 'toolFilter', 'help', 'settings'] satisfies AppAction[]) {
    unregisterFns.push(on(action, () => dispatched.push(action)));
  }

  const Host = defineComponent({
    setup() {
      useShortcuts();
      return () => h('div');
    },
  });
  mounted.push(mount(Host, { attachTo: document.body, global: { plugins: [router] } }));
  return { dispatched, router };
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((wrapper) => wrapper.unmount());
  mounted = [];
  unregisterFns.forEach((off) => off());
  unregisterFns = [];
});

describe('useShortcuts: bindings dispatch the named AppAction', () => {
  it('Mod+K dispatches "search"', async () => {
    const { dispatched } = await mountWith('windows');
    fireKeydown({ key: 'k', ctrlKey: true });
    expect(dispatched).toEqual(['search']);
  });

  it('Mod+T dispatches "toolFilter"', async () => {
    const { dispatched } = await mountWith('windows');
    fireKeydown({ key: 't', ctrlKey: true });
    expect(dispatched).toEqual(['toolFilter']);
  });

  it('Mod+? dispatches "help"', async () => {
    const { dispatched } = await mountWith('windows');
    fireKeydown({ key: '?', ctrlKey: true });
    expect(dispatched).toEqual(['help']);
  });

  it('Mod+, dispatches "settings"', async () => {
    const { dispatched } = await mountWith('windows');
    fireKeydown({ key: ',', ctrlKey: true });
    expect(dispatched).toEqual(['settings']);
  });

  it('a bare key with no modifier at all does not dispatch anything', async () => {
    const { dispatched } = await mountWith('windows');
    fireKeydown({ key: 'k' });
    expect(dispatched).toEqual([]);
  });
});

describe('useShortcuts: Mod is platform-aware', () => {
  it('on macOS, metaKey (⌘) is Mod -- ctrlKey alone does not count', async () => {
    const { dispatched } = await mountWith('macos');
    fireKeydown({ key: 'k', ctrlKey: true });
    expect(dispatched).toEqual([]);

    fireKeydown({ key: 'k', metaKey: true });
    expect(dispatched).toEqual(['search']);
  });

  it('on Windows/Linux, ctrlKey is Mod -- metaKey alone does not count', async () => {
    const { dispatched } = await mountWith('linux');
    fireKeydown({ key: 'k', metaKey: true });
    expect(dispatched).toEqual([]);

    fireKeydown({ key: 'k', ctrlKey: true });
    expect(dispatched).toEqual(['search']);
  });
});

describe('useShortcuts: real handlers for the stores that already exist', () => {
  it('Mod+= / Mod+- / Mod+0 drive the real theme store font scale through a real update_prefs round trip', async () => {
    let current = defaultPrefs();
    mockIPC((cmd, args) => {
      if (cmd === 'get_prefs') return current;
      if (cmd === 'update_prefs') {
        const patch = (args as { patch: Record<string, unknown> }).patch;
        current = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
        return current;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    await mountWith('windows');
    await usePrefsStore().load();
    const themeStore = useThemeStore();
    expect(themeStore.fontScale).toBe(1.0);

    fireKeydown({ key: '=', ctrlKey: true });
    await flushPromises();
    expect(themeStore.fontScale).toBe(1.05);

    // Awaited individually, not fired back to back: increaseFontScale/decreaseFontScale read
    // the current value and write it back without a lock, so two calls fired before either
    // round trip resolves would both compute from the same stale value (confirmed for real).
    fireKeydown({ key: '-', ctrlKey: true });
    await flushPromises();
    fireKeydown({ key: '-', ctrlKey: true });
    await flushPromises();
    expect(themeStore.fontScale).toBeCloseTo(0.95);

    fireKeydown({ key: '0', ctrlKey: true });
    await flushPromises();
    expect(themeStore.fontScale).toBe(1.0);
  });

  it('Mod+[ / Mod+] drive the real router back/forward', async () => {
    const { router } = await mountWith('windows');
    router.addRoute({ path: '/elsewhere', name: 'elsewhere', component: { render: () => null } });
    await router.push('/elsewhere');
    expect(router.currentRoute.value.path).toBe('/elsewhere');

    fireKeydown({ key: '[', ctrlKey: true });
    await flushPromises();
    expect(router.currentRoute.value.path).toBe('/');

    fireKeydown({ key: ']', ctrlKey: true });
    await flushPromises();
    expect(router.currentRoute.value.path).toBe('/elsewhere');
  });

  it('Mod+, navigates to the real settings route', async () => {
    const { router } = await mountWith('windows');

    fireKeydown({ key: ',', ctrlKey: true });
    await flushPromises();

    expect(router.currentRoute.value.name).toBe('settings');
  });
});
