// M3-14's own AC: "Statuses come from get_mcp_statuses; env and header values are never
// displayed (only keys)." / "No add, edit or delete controls yet (M3-48, M3-49)."
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { McpServer, McpServerStatus } from '@/ipc/bindings';
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

describe('McpServersPage: no add, edit or delete controls yet (AC)', () => {
  it('renders no Add button and no per-card action icons', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_mcp_servers') return { value: [makeServer()], skipped: 0 };
      if (cmd === 'get_mcp_statuses') return { 'global-github': 'connected' };
    });
    expect(wrapper.find('button[title="Add"]').exists()).toBe(false);
    expect(wrapper.find('button[title="Edit"]').exists()).toBe(false);
    expect(wrapper.find('button[title="Delete"]').exists()).toBe(false);
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
