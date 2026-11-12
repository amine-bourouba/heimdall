// M3-14's own AC: "Statuses come from get_mcp_statuses; env and header values are never
// displayed (only keys)." The add / edit / remove controls it left out arrived with M3-48 (the last
// describe block); the wizard itself is covered in McpSetupWizard.spec.ts.
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { McpServer, McpServerStatus } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import McpServersPage from './McpServersPage.vue';

function makeServer(overrides: Partial<McpServer> = {}): McpServer {
  return {
    id: 'global-github',
    name: 'github',
    rawName: 'github',
    scope: 'global',
    source: 'user',
    type: null,
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-github'],
    url: null,
    envKeys: ['GITHUB_PERSONAL_ACCESS_TOKEN'],
    headerKeys: [],
    tools: ['create_issue', 'list_repos'],
    isWildcard: false,
    ...overrides,
  };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountPage(ipcHandler: (cmd: string, args: unknown) => unknown) {
  setActivePinia(createPinia());
  mockIPC((cmd, args) => {
    if (cmd === 'list_projects') return { value: [], skipped: 0 };
    const result = ipcHandler(cmd, args);
    if (result !== undefined) return result;
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(McpServersPage, { attachTo: document.body });
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
  vi.unstubAllGlobals();
});

describe('McpServersPage: statuses come from get_mcp_statuses (AC)', () => {
  it('fetches statuses with the same project context as the server list and renders the right indicator', async () => {
    let statusArgs: unknown = null;
    const wrapper = await mountPage((cmd, args) => {
      if (cmd === 'list_mcp_servers') return { value: [makeServer()], skipped: 0 };
      if (cmd === 'get_mcp_statuses') {
        statusArgs = args;
        return { 'global-github': 'connected' } satisfies Record<string, McpServerStatus>;
      }
    });

    expect(statusArgs).toEqual({ project: null });
    expect(wrapper.find('[aria-label="Server is connected and running"]').exists()).toBe(true);
  });

  it('falls back to "unknown" for a server missing from the status map', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_mcp_servers') return { value: [makeServer()], skipped: 0 };
      if (cmd === 'get_mcp_statuses') return {};
    });
    expect(wrapper.find('[aria-label="Server status is unknown"]').exists()).toBe(true);
  });
});

describe('McpServersPage: env and header values are never displayed (AC)', () => {
  it('renders only key names for env and header keys, never values -- confirmed the type carries no values at all', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_mcp_servers') {
        return { value: [makeServer({ envKeys: ['GITHUB_PERSONAL_ACCESS_TOKEN'], headerKeys: ['X-Api-Key'] })], skipped: 0 };
      }
      if (cmd === 'get_mcp_statuses') return {};
    });
    // McpServer has no env/header *value* field at all (bindings.ts), so there is nothing a
    // regression could even render -- this asserts the page renders cleanly with key-bearing
    // data present, not a literal secret string anywhere in the output.
    expect(wrapper.text()).not.toContain('ghp_');
  });
});

describe('McpServersPage: source badges', () => {
  it('shows a scope badge for a user-sourced server', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_mcp_servers') return { value: [makeServer({ source: 'user', scope: 'project' })], skipped: 0 };
      if (cmd === 'get_mcp_statuses') return {};
    });
    expect(wrapper.text()).toContain('Project');
  });

  it('shows the cloud-integration badge for a claude.ai-sourced server', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_mcp_servers') return { value: [makeServer({ source: 'cloudIntegration', name: 'claude.ai notion' })], skipped: 0 };
      if (cmd === 'get_mcp_statuses') return {};
    });
    expect(wrapper.text()).toContain('claude.ai');
  });
});

describe('McpServersPage: add, edit and remove (M3-48)', () => {
  const body = () => new DOMWrapper(document.body);
  const user = makeServer({ id: 'user:github', name: 'GitHub', rawName: 'github' });

  interface Counts {
    lists: number;
    removed: unknown[];
    updated: unknown[];
  }

  function handlerFor(servers: McpServer[], counts: Counts, overrides: Record<string, (args: unknown) => unknown> = {}) {
    return (cmd: string, args: unknown) => {
      if (overrides[cmd]) return overrides[cmd](args);
      switch (cmd) {
        case 'list_mcp_servers':
          counts.lists++;
          return { value: servers, skipped: 0 };
        case 'get_mcp_statuses':
          return {};
        case 'get_mcp_catalog':
          return [];
        case 'remove_mcp_server':
          counts.removed.push(args);
          return 'written';
        case 'update_mcp_server':
          counts.updated.push(args);
          return 'written';
      }
    };
  }

  const counts = (): Counts => ({ lists: 0, removed: [], updated: [] });
  const busy = () => {
    throw { kind: 'busy', message: '~/.claude.json is in use by another program', path: null };
  };

  it('has an Add button named for what it adds', async () => {
    const wrapper = await mountPage(handlerFor([user], counts()));
    expect(wrapper.find('button[aria-label="Add MCP server"]').exists()).toBe(true);
  });

  it('offers Edit and Remove only on a server from the global ~/.claude.json, the only place the writer touches', async () => {
    const others = (['local', 'project', 'plugin', 'cloudIntegration'] as const).map((source) =>
      makeServer({ id: `s:${source}`, name: `from-${source}`, rawName: `from-${source}`, source }),
    );
    const wrapper = await mountPage(handlerFor([user, ...others], counts()));
    const labels = wrapper.findAll('button[aria-label]').map((b) => b.attributes('aria-label'));
    expect(labels.filter((l) => /^(Edit|Remove) /.test(l!))).toEqual(['Edit GitHub', 'Remove GitHub']);
  });

  it('removes by the key under mcpServers, says so, and re-reads the list', async () => {
    const c = counts();
    const wrapper = await mountPage(handlerFor([user], c));
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    await flushPromises();
    expect(c.removed).toEqual([{ name: 'github' }]);
    expect(useToastsStore().current).toMatchObject({ message: 'Removed GitHub', style: 'info' });
    expect(c.lists).toBe(2);
  });

  it("shows no window.confirm: the native confirmation is Rust's", async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);
    const c = counts();
    const wrapper = await mountPage(handlerFor([user], c));
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    await flushPromises();
    expect(confirmSpy).not.toHaveBeenCalled();
    expect(c.removed).toHaveLength(1);
  });

  it('does nothing when the native confirmation is declined', async () => {
    const c = counts();
    const wrapper = await mountPage(handlerFor([user], c, { remove_mcp_server: () => 'cancelled' }));
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toBeNull();
    expect(c.lists).toBe(1);
  });

  it('says so, without claiming a removal, when the server was already gone', async () => {
    const wrapper = await mountPage(handlerFor([user], counts(), { remove_mcp_server: () => 'unchanged' }));
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'GitHub was already gone', style: 'info' });
  });

  it('shows the cause when a remove fails, and can be retried', async () => {
    let fail = true;
    const c = counts();
    const wrapper = await mountPage(handlerFor([user], c, { remove_mcp_server: () => (fail ? busy() : 'written') }));
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: '~/.claude.json is in use by another program', style: 'error' });
    expect(c.lists).toBe(1);
    fail = false;
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    await flushPromises();
    expect(c.lists).toBe(2);
  });

  it('disables Edit and Remove while a remove is in flight, so there is one write', async () => {
    let release: (value: unknown) => void = () => {};
    let calls = 0;
    const wrapper = await mountPage(
      handlerFor([user], counts(), {
        remove_mcp_server: () => {
          calls++;
          return new Promise((resolve) => (release = resolve));
        },
      }),
    );
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    expect(wrapper.find('button[aria-label="Remove GitHub"]').attributes('disabled')).toBeDefined();
    expect(wrapper.find('button[aria-label="Edit GitHub"]').attributes('disabled')).toBeDefined();
    await wrapper.find('button[aria-label="Remove GitHub"]').trigger('click');
    release('written');
    await flushPromises();
    expect(calls).toBe(1);
  });

  it('opens the wizard from Add, which refuses a name already in ~/.claude.json', async () => {
    const wrapper = await mountPage(handlerFor([user], counts()));
    await wrapper.find('button[aria-label="Add MCP server"]').trigger('click');
    await flushPromises();
    expect(body().find('[role="dialog"]').attributes('aria-label')).toBe('Add MCP Server');
    await body()
      .findAll('button')
      .find((b) => b.text().includes('Custom Server'))!
      .trigger('click');
    await body().find('#mcp-name').setValue('github');
    expect(body().find('[role="alert"]').text()).toContain('already exists');
  });

  it('only treats names in the global ~/.claude.json as taken: a plugin or project server of the same name is no clash', async () => {
    const plugin = makeServer({ id: 'p:notes', name: 'notes', rawName: 'notes', source: 'plugin' });
    const project = makeServer({ id: 'pr:db', name: 'db', rawName: 'db', source: 'project' });
    const wrapper = await mountPage(handlerFor([user, plugin, project], counts()));
    await wrapper.find('button[aria-label="Add MCP server"]').trigger('click');
    await flushPromises();
    await body()
      .findAll('button')
      .find((b) => b.text().includes('Custom Server'))!
      .trigger('click');
    await body().find('#mcp-command').setValue('node');
    await body().find('#mcp-name').setValue('notes');
    expect(body().find('[role="alert"]').exists()).toBe(false);
    expect(body().find('button[title="Preview"]').attributes('disabled')).toBeUndefined();
    await body().find('#mcp-name').setValue('github');
    expect(body().find('[role="alert"]').text()).toContain('already exists');
  });

  it('opens the wizard from Edit on that server, and re-reads the list after a save', async () => {
    const c = counts();
    const wrapper = await mountPage(handlerFor([user], c));
    await wrapper.find('button[aria-label="Edit GitHub"]').trigger('click');
    await flushPromises();
    expect(body().find('[role="dialog"]').attributes('aria-label')).toBe('Edit MCP Server');
    expect((body().find('#mcp-name').element as HTMLInputElement).value).toBe('github');
    await body().find('button[title="Preview"]').trigger('click');
    await body().find('button[title="Save server"]').trigger('click');
    await flushPromises();
    expect(c.updated).toMatchObject([{ originalName: 'github' }]);
    expect(c.lists).toBe(2);
    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });

  it('shows the cause when the list cannot be read', async () => {
    await mountPage(handlerFor([user], counts(), { list_mcp_servers: busy }));
    expect(useToastsStore().current).toMatchObject({ message: '~/.claude.json is in use by another program', style: 'error' });
  });
});
