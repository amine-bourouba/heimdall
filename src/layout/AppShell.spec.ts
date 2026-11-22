// M3-58: the shell listens for `navigate`, which `open_in_main_window` emits when the tray (or its Linux
// menu) asks for a session or screen; nothing did before, so the main window came forward and stayed put.
// Children are stubbed: this checks the shell's own wiring, not the sidebar or the pages.
import { defineComponent, h } from 'vue';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { events } from '@/ipc/bindings';
import type { Prefs } from '@/ipc/bindings';
import AppShell from './AppShell.vue';

const Blank = defineComponent({ render: () => h('div') });

// Plain sync stub routes (one per route *name* `navigationItems.ts` targets), as Sidebar.spec.ts does.
function makeRouter() {
  return createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/', redirect: '/sessions' },
      { path: '/sessions', name: 'sessions', component: Blank },
      { path: '/analytics', name: 'analytics', component: Blank },
      { path: '/todos', name: 'todos', component: Blank },
      { path: '/history', name: 'history', component: Blank },
      { path: '/plans', name: 'plans', component: Blank },
      { path: '/memory', name: 'memory', component: Blank },
      { path: '/config/:kind', name: 'config-list', component: Blank },
      { path: '/config/:kind/:scope/:slug', name: 'config-detail', component: Blank },
      { path: '/config/hooks', name: 'hooks', component: Blank },
      { path: '/config/mcp', name: 'mcp', component: Blank },
      { path: '/config/plugins', name: 'plugins', component: Blank },
      { path: '/config/models', name: 'models', component: Blank },
      { path: '/sessions/:projectId/:sessionId?', name: 'session-detail', component: Blank },
    ],
  });
}

const prefs: Prefs = {
  schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1, sessionLayout: 'grid',
  configProjectPath: null, configLayouts: {}, showAgentSessions: false, textEditor: 'vscode', preferredTerminal: 'terminal',
  openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true, autoExpandBlocks: true,
  parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [],
};

let mounted: ReturnType<typeof mount>[] = [];

async function mountShell() {
  setActivePinia(createPinia());
  mockIPC(
    (cmd) => {
      if (cmd === 'get_app_info') return { version: '0', identifier: 'x', os: 'windows', arch: 'x86_64', claudeHomeFound: true };
      if (cmd === 'get_prefs') return prefs;
      return null;
    },
    { shouldMockEvents: true },
  );
  const router = makeRouter();
  await router.push('/todos');
  const wrapper = mount(AppShell, {
    global: {
      plugins: [router],
      stubs: { Sidebar: true, Toolbar: true, AppMenu: true, SearchOverlay: true, Onboarding: true, ShortcutHelp: true, ToastOverlay: true },
    },
    attachTo: document.body,
  });
  mounted.push(wrapper);
  await flushPromises();
  await vi.waitFor(() => expect(wrapper.vm).toBeTruthy());
  await flushPromises();
  return { wrapper, router };
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('AppShell: the navigate event (M3-58)', () => {
  it('opens a session in its detail view', async () => {
    const { router } = await mountShell();
    await events.navigate.emit({ target: { kind: 'session', projectId: 'proj-1', sessionId: 'sess-2' } });
    await flushPromises();
    expect(router.currentRoute.value.fullPath).toBe('/sessions/proj-1/sess-2');
  });

  it('opens a screen at that screen\'s route', async () => {
    const { router } = await mountShell();
    await events.navigate.emit({ target: { kind: 'screen', screen: 'history' } });
    await flushPromises();
    expect(router.currentRoute.value.name).toBe('history');
  });

  it('follows each request in turn, wherever the window already was', async () => {
    const { router } = await mountShell();
    await events.navigate.emit({ target: { kind: 'screen', screen: 'mcpServers' } });
    await flushPromises();
    expect(router.currentRoute.value.name).toBe('mcp');
    await events.navigate.emit({ target: { kind: 'session', projectId: 'p', sessionId: 's' } });
    await flushPromises();
    expect(router.currentRoute.value.name).toBe('session-detail');
  });

  it('stops listening when the shell is unmounted', async () => {
    const { wrapper, router } = await mountShell();
    // Not the current route: Vue Router resets that to its start location when the last app unmounts.
    const push = vi.spyOn(router, 'push');
    wrapper.unmount();
    mounted = [];
    await events.navigate.emit({ target: { kind: 'screen', screen: 'history' } });
    await flushPromises();
    expect(push).not.toHaveBeenCalled();
  });
});
