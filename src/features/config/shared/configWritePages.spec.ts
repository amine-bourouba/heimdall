// M3-50's wiring on the three template screens, checked once for all of them so they cannot drift
// apart: the "+" creates a template of the screen's own kind and reloads, and the detail view's
// trash button deletes the reference the backend resolved (not one rebuilt from the route) and
// returns to the list. The create/delete logic itself is in useConfigWriteActions.spec.ts.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory, type Router } from 'vue-router';
import type { Component } from 'vue';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ConfigItemDetail, ConfigItemSummary, ConfigKind, ConfigRef } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import CommandsPage from '../commands/CommandsPage.vue';
import SkillsPage from '../skills/SkillsPage.vue';
import OutputStylesPage from '../output-styles/OutputStylesPage.vue';
import ConfigSkeleton from './ConfigSkeleton.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

interface Screen {
  label: string;
  page: Component;
  kind: ConfigKind;
  addLabel: string;
  name: string;
  slug: string;
}

const SCREENS: Screen[] = [
  { label: 'CommandsPage', page: CommandsPage, kind: 'command', addLabel: 'New command', name: '/deploy', slug: 'deploy' },
  { label: 'SkillsPage', page: SkillsPage, kind: 'skill', addLabel: 'New skill', name: 'deploy', slug: 'deploy' },
  { label: 'OutputStylesPage', page: OutputStylesPage, kind: 'outputStyle', addLabel: 'New output style', name: 'concise', slug: 'concise' },
];

const busy = () => {
  throw { kind: 'busy', message: 'the file is in use by another program', path: null };
};

let mounted: ReturnType<typeof mount>[] = [];

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.unstubAllGlobals();
});

interface Seen {
  lists: number;
  created: unknown[];
  deleted: unknown[];
}

async function mountScreen(screen: Screen, overrides: Partial<Record<string, (args: unknown) => unknown>> = {}, path = `/config/${screen.kind}`, detailRef?: ConfigRef) {
  setActivePinia(createPinia());
  const seen: Seen = { lists: 0, created: [], deleted: [] };
  const summary: ConfigItemSummary = { ref: { kind: screen.kind, project: null, slug: screen.slug }, name: screen.name, description: 'd', scope: 'global', meta: {} };
  const detail: ConfigItemDetail = { ...summary, ref: detailRef ?? summary.ref, body: '# Body', displayPath: '' };
  mockIPC((cmd, args) => {
    if (overrides[cmd]) return overrides[cmd]!(args);
    switch (cmd) {
      case 'list_projects':
        return { value: [], skipped: 0 };
      case 'list_config_items':
        seen.lists++;
        return { value: [summary], skipped: 0 };
      case 'get_config_item':
        return detail;
      case 'create_config_template':
        seen.created.push(args);
        return { kind: screen.kind, project: null, slug: 'new-thing' };
      case 'delete_config_item':
        seen.deleted.push(args);
        return 'written';
      default:
        throw new Error(`unexpected command: ${cmd}`);
    }
  });
  const router: Router = createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/config/:kind', name: 'config-list', component: screen.page },
      { path: '/config/:kind/:scope/:slug', name: 'config-detail', component: screen.page },
    ],
  });
  await router.push(path);
  const wrapper = mount(screen.page, { global: { plugins: [router] }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return { wrapper, router, seen, toasts: useToastsStore() };
}

describe.each(SCREENS)('$label: create from template', (screen) => {
  it('has a labelled add button that creates a global template of its own kind, then reloads the list', async () => {
    const { wrapper, seen, toasts } = await mountScreen(screen);
    expect(seen.lists).toBe(1);

    await wrapper.find(`button[aria-label="${screen.addLabel}"]`).trigger('click');
    await flushPromises();

    expect(seen.created).toEqual([{ kind: screen.kind, project: null }]);
    expect(seen.lists).toBe(2);
    expect(toasts.current).toMatchObject({ style: 'success' });
    expect(toasts.current?.message).toContain('new-thing');
  });

  it('shows the cause and does not reload when the template cannot be written', async () => {
    const { wrapper, seen, toasts } = await mountScreen(screen, { create_config_template: busy });

    await wrapper.find(`button[aria-label="${screen.addLabel}"]`).trigger('click');
    await flushPromises();

    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(seen.lists).toBe(1);
  });
});

describe.each(SCREENS)('$label: delete from the detail view', (screen) => {
  const detailPath = (s: Screen) => `/config/${s.kind}/global/${s.slug}`;

  it('has no delete button on the list', async () => {
    const { wrapper } = await mountScreen(screen);
    expect(wrapper.find('button[title="Delete"]').exists()).toBe(false);
  });

  it('deletes the reference the backend resolved, says so, returns to the list and reloads it', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    // A reference that differs from the one the route would rebuild (project null): deleting the
    // rebuilt one would delete a different file than the one on screen.
    const resolved: ConfigRef = { kind: screen.kind, project: { kind: 'project', projectId: 'p9' }, slug: screen.slug };
    const { wrapper, router, seen, toasts } = await mountScreen(screen, {}, detailPath(screen), resolved);
    expect(wrapper.text()).toContain('Body');
    const listsBefore = seen.lists;

    await wrapper.find('button[title="Delete"]').trigger('click');
    await flushPromises();

    expect(seen.deleted).toEqual([{ configRef: resolved }]);
    expect(toasts.current).toMatchObject({ message: `Deleted ${screen.name}`, style: 'info' });
    expect(router.currentRoute.value.name).toBe('config-list');
    expect(seen.lists).toBeGreaterThan(listsBefore);
  });

  it('asks first, and a declined confirmation sends nothing and stays on the item', async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);
    const { wrapper, router, seen } = await mountScreen(screen, {}, detailPath(screen));

    await wrapper.find('button[title="Delete"]').trigger('click');
    await flushPromises();

    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(confirmSpy.mock.calls[0][0]).toContain(`"${screen.name}"`);
    expect(seen.deleted).toEqual([]);
    expect(router.currentRoute.value.name).toBe('config-detail');
  });

  it('stays on the item and shows the cause when the delete fails', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    const { wrapper, router, toasts } = await mountScreen(screen, { delete_config_item: busy }, detailPath(screen));

    await wrapper.find('button[title="Delete"]').trigger('click');
    await flushPromises();

    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(router.currentRoute.value.name).toBe('config-detail');
    expect(wrapper.text()).toContain('Body');
  });
});

describe.each(SCREENS)('$label: loading', (screen) => {
  it('shows the cause and leaves the skeleton when the list cannot be loaded', async () => {
    const { wrapper, toasts } = await mountScreen(screen, { list_config_items: busy });

    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(wrapper.findComponent(ConfigSkeleton).exists()).toBe(false);
  });

  it('shows the cause when the item cannot be opened', async () => {
    const { toasts } = await mountScreen(screen, { get_config_item: busy }, `/config/${screen.kind}/global/${screen.slug}`);

    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
  });
});
