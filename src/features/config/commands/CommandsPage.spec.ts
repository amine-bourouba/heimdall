// M3-12's own AC: "Copy /name works for commands (CommandsListView.swift:297-300)." / "No
// create, edit or delete controls yet (M3-50)."
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createWebHashHistory, type Router } from 'vue-router';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ConfigItemDetail, ConfigItemSummary } from '@/ipc/bindings';
import CommandsPage from './CommandsPage.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function makeSummary(overrides: Partial<ConfigItemSummary> = {}): ConfigItemSummary {
  return { ref: { kind: 'command', project: null, slug: 'deploy' }, name: '/deploy', description: 'Deploy the app', scope: 'global', meta: {}, ...overrides };
}

function makeDetail(overrides: Partial<ConfigItemDetail> = {}): ConfigItemDetail {
  return {
    ref: { kind: 'command', project: null, slug: 'deploy' },
    name: '/deploy',
    description: 'Deploy the app',
    scope: 'global',
    meta: {},
    body: '# Deploy\n\nRuns the deploy script.',
    displayPath: '~/.claude/commands/deploy.md',
    ...overrides,
  };
}

function makeRouter(): Router {
  return createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/config/:kind', name: 'config-list', component: CommandsPage },
      { path: '/config/:kind/:scope/:slug', name: 'config-detail', component: CommandsPage },
    ],
  });
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountPage(ipcHandler: (cmd: string, args: unknown) => unknown, initialPath = '/config/command') {
  setActivePinia(createPinia());
  mockIPC((cmd, args) => {
    const result = ipcHandler(cmd, args);
    if (result !== undefined) return result;
    throw new Error(`unexpected command: ${cmd}`);
  });
  const router = makeRouter();
  await router.push(initialPath);
  const wrapper = mount(CommandsPage, { global: { plugins: [router] }, attachTo: document.body });
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

describe('CommandsPage: list', () => {
  it('lists commands for the current project context', async () => {
    let args: unknown = null;
    const { wrapper } = await mountPage((cmd, a) => {
      if (cmd === 'list_config_items') {
        args = a;
        return { value: [makeSummary()], skipped: 0 };
      }
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });
    expect(args).toEqual({ kind: 'command', project: null });
    expect(wrapper.text()).toContain('/deploy');
    expect(wrapper.text()).toContain('Deploy the app');
  });

  it('shows "No commands found" when the list is empty', async () => {
    const { wrapper } = await mountPage((cmd) => {
      if (cmd === 'list_config_items') return { value: [], skipped: 0 };
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });
    expect(wrapper.text()).toContain('No commands found');
  });

  it('renders no Add button (AC: no create/edit/delete controls yet)', async () => {
    const { wrapper } = await mountPage((cmd) => {
      if (cmd === 'list_config_items') return { value: [makeSummary()], skipped: 0 };
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });
    expect(wrapper.find('button[title="Add"]').exists()).toBe(false);
  });
});

describe('CommandsPage: copy /name (AC)', () => {
  it('copies the full "/name" string, including the prefix', async () => {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const { wrapper } = await mountPage((cmd) => {
      if (cmd === 'list_config_items') return { value: [makeSummary({ name: '/deploy-prod' })], skipped: 0 };
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });

    await wrapper.find('button[title="Copy /name"]').trigger('click');

    expect(writeText).toHaveBeenCalledWith('/deploy-prod');
  });
});

describe('CommandsPage: opening and closing the detail view', () => {
  it('clicking a card navigates to the detail route and fetches get_config_item', async () => {
    let detailArgs: unknown = null;
    const { wrapper, router } = await mountPage((cmd, a) => {
      if (cmd === 'list_config_items') return { value: [makeSummary()], skipped: 0 };
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
      if (cmd === 'get_config_item') {
        detailArgs = a;
        return makeDetail();
      }
    });

    await wrapper.find('[data-testid="config-item-card"]').trigger('click');
    await flushPromises();

    expect(router.currentRoute.value.name).toBe('config-detail');
    expect(detailArgs).toEqual({ configRef: { kind: 'command', project: null, slug: 'deploy' } });
    expect(wrapper.text()).toContain('Runs the deploy script.');
  });

  it('Close navigates back to the list route', async () => {
    const { wrapper, router } = await mountPage(
      (cmd) => {
        if (cmd === 'list_config_items') return { value: [], skipped: 0 };
        if (cmd === 'list_projects') return { value: [], skipped: 0 };
        if (cmd === 'get_config_item') return makeDetail();
      },
      '/config/command/global/deploy',
    );
    expect(wrapper.text()).toContain('Runs the deploy script.');

    await wrapper.find('button[title="Close"]').trigger('click');
    await flushPromises();

    expect(router.currentRoute.value.name).toBe('config-list');
  });
});

describe('CommandsPage: filter', () => {
  it('narrows the list by name/description and shows the no-match empty state', async () => {
    const { wrapper } = await mountPage((cmd) => {
      if (cmd === 'list_config_items') {
        return {
          value: [makeSummary({ name: '/deploy', ref: { kind: 'command', project: null, slug: 'deploy' } }), makeSummary({ name: '/test', description: 'Run tests', ref: { kind: 'command', project: null, slug: 'test' } })],
          skipped: 0,
        };
      }
      if (cmd === 'list_projects') return { value: [], skipped: 0 };
    });
    expect(wrapper.text()).toContain('/test');

    await wrapper.find('input[aria-label="Find in Commands…"]').setValue('zzzznomatch');
    await flushPromises();

    expect(wrapper.text()).toContain('No commands match "zzzznomatch"');
  });
});
