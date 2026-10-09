import { defineComponent, h } from 'vue';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { events } from '@/ipc/bindings';
import type { Prefs, SidebarCounts } from '@/ipc/bindings';
import { navigationItems } from '@/router/navigationItems';
import { usePrefsStore } from '@/stores/prefs';
import Sidebar from './Sidebar.vue';

const Blank = defineComponent({ render: () => h('div') });

// Plain sync stub routes, one per real route *name* `navigationItems.ts` actually targets --
// not `@/router/routes`'s own real lazy `() => import(...)` pages. Reusing those lazy loaders
// here made `router.push` resolve a tick later than `flushPromises()` reaches (confirmed for
// real: an `afterEach` navigation hook fired correctly, but only after the test's own
// assertions had already run), the same async-import delay `useNavigation.spec.ts` already
// sidesteps with its own plain `Blank` routes rather than the app's real page components.
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
      { path: '/config/hooks', name: 'hooks', component: Blank },
      { path: '/config/mcp', name: 'mcp', component: Blank },
      { path: '/config/plugins', name: 'plugins', component: Blank },
      { path: '/config/models', name: 'models', component: Blank },
    ],
  });
}

function defaultPrefs(): Prefs {
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
  };
}

function zeroCounts(): SidebarCounts {
  return {
    todos: 0,
    history: 0,
    commands: 0,
    skills: 0,
    plans: 0,
    memory: 0,
    mcpServers: 0,
    models: 0,
    hooks: 0,
    subAgents: 0,
    plugins: 0,
    outputStyles: 0,
  };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountSidebar(getSidebarCounts: (project: unknown) => SidebarCounts) {
  setActivePinia(createPinia());
  usePrefsStore().prefs = defaultPrefs();
  const router = makeRouter();
  await router.push('/');

  mockIPC((cmd, args) => {
    if (cmd === 'get_sidebar_counts') return getSidebarCounts((args as { project: unknown }).project);
    throw new Error(`unexpected command: ${cmd}`);
  }, { shouldMockEvents: true });

  const wrapper = mount(Sidebar, { attachTo: document.body, global: { plugins: [router] } });
  mounted.push(wrapper);
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

describe('Sidebar: items', () => {
  it('renders every navigationItems entry, in its real order', async () => {
    const { wrapper } = await mountSidebar(() => zeroCounts());
    const labels = wrapper.findAll('a').map((a) => a.text());
    expect(labels.length).toBe(navigationItems.length);
    navigationItems.forEach((item, index) => {
      expect(labels[index]).toContain(item.label);
    });
  });

  it('shows no count badge for an item whose count is 0', async () => {
    const { wrapper } = await mountSidebar(() => zeroCounts());
    expect(wrapper.text()).not.toMatch(/\d/);
  });

  it('shows the real count next to the matching item', async () => {
    const { wrapper } = await mountSidebar(() => ({ ...zeroCounts(), todos: 4, subAgents: 2 }));
    const todosLink = wrapper.findAll('a').find((a) => a.text().includes(navigationItems.find((i) => i.id === 'todos')!.label))!;
    expect(todosLink.text()).toContain('4');
    const agentsLink = wrapper.findAll('a').find((a) => a.text().includes(navigationItems.find((i) => i.id === 'subAgents')!.label))!;
    expect(agentsLink.text()).toContain('2');
  });

  it('passes a pickedFolder context when prefs.configProjectPath is set, null otherwise', async () => {
    const seen: unknown[] = [];
    await mountSidebar((project) => {
      seen.push(project);
      return zeroCounts();
    });
    expect(seen[0]).toBeNull();

    mounted.forEach((w) => w.unmount());
    mounted = [];
    setActivePinia(createPinia());
    usePrefsStore().prefs = { ...defaultPrefs(), configProjectPath: '/some/project' };
    const router = makeRouter();
    await router.push('/');
    mockIPC((cmd, args) => {
      if (cmd === 'get_sidebar_counts') {
        seen.push((args as { project: unknown }).project);
        return zeroCounts();
      }
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    mounted.push(mount(Sidebar, { attachTo: document.body, global: { plugins: [router] } }));
    await flushPromises();
    expect(seen[1]).toEqual({ kind: 'pickedFolder' });
  });
});

describe('Sidebar: counts refresh on data-changed', () => {
  it('re-fetches counts when a watched area changes', async () => {
    const getCounts = vi.fn(() => zeroCounts());
    await mountSidebar(getCounts);
    expect(getCounts).toHaveBeenCalledTimes(1);

    await events.dataChanged.emit([{ area: 'todos', key: null }]);
    await flushPromises();
    expect(getCounts).toHaveBeenCalledTimes(2);
  });

  it('does not re-fetch for an unrelated area', async () => {
    const getCounts = vi.fn(() => zeroCounts());
    await mountSidebar(getCounts);
    expect(getCounts).toHaveBeenCalledTimes(1);

    await events.dataChanged.emit([{ area: 'sessions', key: null }]);
    await flushPromises();
    expect(getCounts).toHaveBeenCalledTimes(1);
  });
});

describe('Sidebar: keyboard navigation', () => {
  function fireKeydown(init: KeyboardEventInit) {
    document.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }));
  }

  it('ArrowDown highlights the next item, Enter navigates to it', async () => {
    const { router } = await mountSidebar(() => zeroCounts());

    fireKeydown({ key: 'ArrowDown' });
    fireKeydown({ key: 'Enter' });
    await flushPromises();

    // Every real navigationItems entry targets a named route object (never a bare path string).
    expect(router.currentRoute.value.name).toBe((navigationItems[1].to as { name: string }).name);
  });
});
