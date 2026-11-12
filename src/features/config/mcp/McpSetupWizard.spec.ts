// M3-48's acceptance criteria, through the real dialog: custom arguments are an ordered list and are
// never split on spaces; the preview masks every env and header value completely, custom servers
// included; editing sends `keep` for a secret nobody touched (and shows no stored value anywhere).
// The pure logic is covered in wizardModel.spec.ts; this file checks what the user sees and what is sent.
//
// Every query goes through `document.body` (the root is a `<Teleport to="body">`) and `onClose` /
// `onSaved` are `vi.fn()` callback props, for the reasons HookForm.spec.ts documents.
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { McpCatalogEntry, McpServer, McpServerInput, TargetOs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { useToastsStore } from '@/stores/toasts';
import McpSetupWizard from './McpSetupWizard.vue';

const body = () => new DOMWrapper(document.body);

function key(id: string, overrides: Record<string, unknown> = {}) {
  return { id, label: id, description: `About ${id}`, placeholder: '', isRequired: true, isSensitive: false, ...overrides };
}

const CATALOG: McpCatalogEntry[] = [
  { id: 'github', name: 'GitHub', description: 'Repos and issues', icon: 'arrow.triangle.branch', category: 'developer', command: 'npx', args: ['-y', '@mcp/github'], envKeys: [key('GITHUB_TOKEN', { label: 'Access token', isSensitive: true, placeholder: 'ghp_…' })] },
  { id: 'playwright', name: 'Playwright', description: 'Browser automation', icon: 'hammer', category: 'developer', command: 'npx', args: ['-y', '@mcp/playwright'], envKeys: [] },
  { id: 'slack', name: 'Slack', description: 'Messages', icon: 'tray.full', category: 'productivity', command: 'npx', args: ['-y', '@mcp/slack'], envKeys: [key('SLACK_BOT_TOKEN', { isSensitive: true }), key('SLACK_TEAM_ID')] },
  { id: 'filesystem', name: 'Filesystem', description: 'Local files', icon: 'folder', category: 'filesystem', command: 'npx', args: ['-y', '@mcp/filesystem'], envKeys: [key('ALLOWED_DIRS', { label: 'Allowed directories' })] },
] as McpCatalogEntry[];

function server(overrides: Partial<McpServer> = {}): McpServer {
  return { id: 'user:gh', name: 'Gh', rawName: 'gh', scope: 'global', source: 'user', type: 'stdio', command: 'node', args: ['server.js', '--port 8080'], url: null, envKeys: ['TOKEN', 'OTHER'], headerKeys: [], tools: [], isWildcard: false, ...overrides };
}

interface Sent {
  add: { input: McpServerInput }[];
  update: { originalName: string; input: McpServerInput }[];
  catalogCalls: number;
}

let mounted: ReturnType<typeof mount>[] = [];
let sent: Sent;
let release: (outcome: string) => void = () => {};

interface Options {
  editing?: McpServer | null;
  existingNames?: string[];
  /** What add/update answers: a WriteOutcome, 'error' (a busy file), or 'hold' (never answers until `release`). */
  outcome?: 'written' | 'cancelled' | 'unchanged' | 'error' | 'hold';
  catalog?: 'ok' | 'error';
  os?: TargetOs;
}

function mountWizard({ editing = null, existingNames = [], outcome = 'written', catalog = 'ok', os }: Options = {}) {
  setActivePinia(createPinia());
  if (os) usePlatformStore().os = os;
  sent = { add: [], update: [], catalogCalls: 0 };
  const answer = () => {
    if (outcome === 'error') throw { kind: 'busy', message: '~/.claude.json is in use by another program', path: null };
    if (outcome === 'hold') return new Promise((resolve) => (release = resolve));
    return outcome;
  };
  mockIPC((cmd, args) => {
    switch (cmd) {
      case 'get_mcp_catalog':
        sent.catalogCalls++;
        if (catalog === 'error') throw { kind: 'io', message: 'catalog unavailable', path: null };
        return CATALOG;
      case 'add_mcp_server':
        sent.add.push(args as { input: McpServerInput });
        return answer();
      case 'update_mcp_server':
        sent.update.push(args as { originalName: string; input: McpServerInput });
        return answer();
      default:
        throw new Error(`unexpected command: ${cmd}`);
    }
  });
  const onClose = vi.fn();
  const onSaved = vi.fn();
  const wrapper = mount(McpSetupWizard, { props: { editing, existingNames, onClose, onSaved }, attachTo: document.body });
  mounted.push(wrapper);
  return { wrapper, onClose, onSaved, toasts: useToastsStore() };
}

const settled = async () => {
  await flushPromises();
  await flushPromises();
};

const buttonText = (text: string) => body().findAll('button').find((b) => b.text().includes(text))!;
const button = (title: string) => body().find(`button[title="${title}"]`);
const nameInput = () => body().find('#mcp-name');
const previewText = () => body().find('pre').text();
const argInputs = () => body().findAll('input[aria-label^="Argument"]');

async function click(text: string) {
  await buttonText(text).trigger('click');
  await settled();
}

async function pickCustom() {
  await click('Custom Server');
}

async function pickCatalog(name: string) {
  await buttonText(name).trigger('click');
  await settled();
}

async function addArg(value: string) {
  await click('Add argument');
  await argInputs().at(-1)!.setValue(value);
}

async function addVariable(name: string, value: string) {
  await click('Add variable');
  await body().findAll('input[aria-label="Variable name"]').at(-1)!.setValue(name);
  await body().findAll('input[aria-label^="Value for"]').at(-1)!.setValue(value);
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('McpSetupWizard: step 1, select', () => {
  it('lists Custom Server first, then the catalog by category with what each needs', async () => {
    mountWizard();
    await settled();
    const text = body().text();
    expect(text.indexOf('Custom Server')).toBeLessThan(text.indexOf('Developer Tools'));
    expect(['Developer Tools', 'Productivity', 'Filesystem'].map((c) => text.indexOf(c))).toEqual([...[text.indexOf('Developer Tools'), text.indexOf('Productivity'), text.indexOf('Filesystem')]].sort((a, b) => a - b));
    expect(buttonText('Playwright').text()).toContain('No config needed');
    expect(buttonText('GitHub').text()).toContain('1 var');
    expect(buttonText('Slack').text()).toContain('2 vars');
    expect(body().find('[aria-current="step"]').text()).toContain('Select');
  });

  it('narrows the catalog by name, description or category, and says when nothing matches', async () => {
    mountWizard();
    await settled();
    await body().find('input[type="search"]').setValue('browser');
    expect(body().findAll('button').some((b) => b.text().includes('Playwright'))).toBe(true);
    expect(body().findAll('button').some((b) => b.text().includes('GitHub'))).toBe(false);
    await body().find('input[type="search"]').setValue('zzz');
    expect(body().text()).toContain('No servers match "zzz"');
  });

  it('opens the custom form from Custom Server, and a catalog server\'s from its row', async () => {
    mountWizard();
    await settled();
    await pickCustom();
    expect(body().find('#mcp-command').exists()).toBe(true);
    await click('Back');
    await pickCatalog('GitHub');
    expect((nameInput().element as HTMLInputElement).value).toBe('github');
    expect(body().text()).toContain('npx -y @mcp/github');
    expect(body().text()).toContain('Required');
    expect(body().text()).not.toContain('Add variable');
  });

  it('still lets a custom server be added when the catalog cannot be loaded, and says why', async () => {
    const { toasts } = mountWizard({ catalog: 'error' });
    await settled();
    expect(body().find('[role="alert"]').text()).toContain('catalog could not be loaded');
    expect(toasts.current).toMatchObject({ message: 'catalog unavailable', style: 'error' });
    await pickCustom();
    expect(body().find('#mcp-command').exists()).toBe(true);
  });
});

describe('McpSetupWizard: a custom server', () => {
  async function fillCustom() {
    mountWizard();
    await settled();
    await pickCustom();
    await nameInput().setValue('my-tool');
    await body().find('#mcp-command').setValue('npx');
    await addArg('--config');
    await addArg('/Users/a b/my config.json');
    await addVariable('API_KEY', 'sk-live-SECRET-123');
  }

  it('keeps arguments as an ordered list, an argument with spaces staying one argument (AC1)', async () => {
    await fillCustom();
    await click('Preview');
    expect(JSON.parse(previewText())['my-tool'].args).toEqual(['--config', '/Users/a b/my config.json']);
    await click('Add Server');
    expect(sent.add[0].input.args).toEqual(['--config', '/Users/a b/my config.json']);
  });

  it('masks every env value in the preview, and the secret appears nowhere on the page (AC2)', async () => {
    await fillCustom();
    await click('Preview');
    expect(JSON.parse(previewText())['my-tool']).toEqual({ type: 'stdio', command: 'npx', args: ['--config', '/Users/a b/my config.json'], env: { API_KEY: '********' } });
    expect(document.body.textContent).not.toContain('sk-live-SECRET-123');
    expect(document.body.innerHTML).not.toContain('sk-live-SECRET-123');
    expect(document.body.innerHTML).not.toContain('sk-l');
  });

  it('sends exactly what was entered, the new secret as a set value', async () => {
    await fillCustom();
    await click('Preview');
    await click('Add Server');
    expect(sent.add).toEqual([
      { input: { name: 'my-tool', type: 'stdio', command: 'npx', args: ['--config', '/Users/a b/my config.json'], url: null, env: { API_KEY: { kind: 'set', value: 'sk-live-SECRET-123' } }, headers: {} } },
    ]);
  });

  it('types variable values into password inputs', async () => {
    await fillCustom();
    expect(body().find('input[aria-label^="Value for"]').attributes('type')).toBe('password');
  });

  it('keeps what was typed when going back from the preview', async () => {
    await fillCustom();
    await click('Preview');
    await click('Back');
    expect((nameInput().element as HTMLInputElement).value).toBe('my-tool');
    expect(argInputs().map((i) => (i.element as HTMLInputElement).value)).toEqual(['--config', '/Users/a b/my config.json']);
  });

  it('cannot be previewed until it is valid, and says what is missing', async () => {
    mountWizard();
    await settled();
    await pickCustom();
    expect(button('Preview').attributes('disabled')).toBeDefined();
    expect(body().text()).toContain('Enter a name.');
    expect(body().text()).toContain('Enter the command to run.');
    await nameInput().setValue('x');
    await body().find('#mcp-command').setValue('node');
    expect(button('Preview').attributes('disabled')).toBeUndefined();
  });

  it('refuses a name that is already taken, with an alert, before anything is sent', async () => {
    mountWizard({ existingNames: ['gh'] });
    await settled();
    await pickCustom();
    await nameInput().setValue('gh');
    await body().find('#mcp-command').setValue('node');
    expect(body().find('[role="alert"]').text()).toContain('already exists');
    expect(button('Preview').attributes('disabled')).toBeDefined();
  });

  it('switches to an http server: a URL and headers instead of a command, headers masked too', async () => {
    mountWizard();
    await settled();
    await pickCustom();
    await nameInput().setValue('remote');
    await body().findAll('input[name="mcp-transport"]')[1].setValue(true);
    expect(body().find('#mcp-command').exists()).toBe(false);
    await body().find('#mcp-url').setValue('https://x.example/mcp');
    await click('Add header');
    await body().find('input[aria-label="Header name"]').setValue('Authorization');
    await body().find('input[aria-label^="Value for"]').setValue('Bearer top-secret-token');
    await click('Preview');
    expect(JSON.parse(previewText()).remote).toEqual({ type: 'http', url: 'https://x.example/mcp', headers: { Authorization: '********' } });
    expect(document.body.textContent).not.toContain('top-secret-token');
    await click('Add Server');
    expect(sent.add[0].input).toEqual({ name: 'remote', type: 'http', command: null, args: [], url: 'https://x.example/mcp', env: {}, headers: { Authorization: { kind: 'set', value: 'Bearer top-secret-token' } } });
  });

  it('refuses a URL that is not http or https', async () => {
    mountWizard();
    await settled();
    await pickCustom();
    await nameInput().setValue('remote');
    await body().findAll('input[name="mcp-transport"]')[2].setValue(true);
    await body().find('#mcp-url').setValue('ftp://x.example');
    expect(button('Preview').attributes('disabled')).toBeDefined();
    expect(body().text()).toContain('valid http:// or https:// URL');
  });
});

describe('McpSetupWizard: a catalog server', () => {
  it('needs its required variable, which is typed into a password input, and sends it as a set value', async () => {
    mountWizard();
    await settled();
    await pickCatalog('GitHub');
    expect(button('Preview').attributes('disabled')).toBeDefined();
    const token = body().find('input[type="password"]');
    expect(token.attributes('placeholder')).toBe('ghp_…');
    await token.setValue('ghp_realtoken');
    await click('Preview');
    expect(JSON.parse(previewText()).github).toEqual({ type: 'stdio', command: 'npx', args: ['-y', '@mcp/github'], env: { GITHUB_TOKEN: '********' } });
    expect(document.body.innerHTML).not.toContain('ghp_realtoken');
    await click('Add Server');
    expect(sent.add[0].input.env).toEqual({ GITHUB_TOKEN: { kind: 'set', value: 'ghp_realtoken' } });
  });

  it('masks a variable the catalog does not call sensitive just the same', async () => {
    mountWizard();
    await settled();
    await pickCatalog('Slack');
    await body().find('input[type="password"]').setValue('xoxb-token');
    await body().find('input[type="text"][data-value-input]').setValue('T0123456');
    await click('Preview');
    expect(JSON.parse(previewText()).slack.env).toEqual({ SLACK_BOT_TOKEN: '********', SLACK_TEAM_ID: '********' });
    expect(document.body.textContent).not.toContain('T0123456');
  });

  it('says so when a server needs no variables, and previews and adds it directly', async () => {
    mountWizard();
    await settled();
    await pickCatalog('Playwright');
    expect(body().text()).toContain('No environment variables required');
    await click('Preview');
    await click('Add Server');
    expect(sent.add[0].input).toMatchObject({ name: 'playwright', command: 'npx', args: ['-y', '@mcp/playwright'], env: {} });
  });

  it('turns the filesystem server\'s directories into arguments', async () => {
    mountWizard();
    await settled();
    await pickCatalog('Filesystem');
    await body().find('input[type="text"][data-value-input]').setValue('/home/a, /srv/data');
    await click('Preview');
    await click('Add Server');
    expect(sent.add[0].input.args).toEqual(['-y', '@mcp/filesystem', '/home/a', '/srv/data']);
    expect(sent.add[0].input.env).toEqual({});
  });
});

describe('McpSetupWizard: saving', () => {
  async function toPreview() {
    mountWizard({ outcome: 'written' });
    await settled();
    await pickCatalog('Playwright');
    await click('Preview');
  }

  it('says what it did, tells the page, and closes', async () => {
    const { onSaved, onClose, toasts } = mountWizard();
    await settled();
    await pickCatalog('Playwright');
    await click('Preview');
    await click('Add Server');
    expect(toasts.current).toMatchObject({ message: 'Added playwright', style: 'success' });
    expect(onSaved).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('stays open, writes nothing and says so when the native confirmation is declined', async () => {
    const { onSaved, onClose, toasts } = mountWizard({ outcome: 'cancelled' });
    await settled();
    await pickCatalog('Playwright');
    await click('Preview');
    await click('Add Server');
    expect(body().find('[role="status"]').text()).toContain('Nothing was written');
    expect([onSaved, onClose].map((f) => f.mock.calls.length)).toEqual([0, 0]);
    expect(toasts.current).toBeNull();
  });

  it('shows the cause and stays open on failure, and can be retried', async () => {
    const { onSaved, toasts } = mountWizard({ outcome: 'error' });
    await settled();
    await pickCatalog('Playwright');
    await click('Preview');
    await click('Add Server');
    expect(toasts.current).toMatchObject({ message: '~/.claude.json is in use by another program', style: 'error' });
    expect(onSaved).not.toHaveBeenCalled();
    await click('Add Server');
    expect(sent.add).toHaveLength(2);
  });

  it('sends one write for a double click', async () => {
    mountWizard({ outcome: 'hold' });
    await settled();
    await pickCatalog('Playwright');
    await click('Preview');
    void button('Add server').trigger('click');
    void button('Add server').trigger('click');
    await settled();
    expect(sent.add).toHaveLength(1);
    release('written');
    await settled();
  });

  it('puts the preview and the on-disk note in the preview step', async () => {
    await toPreview();
    expect(body().text()).toContain('~/.claude.json');
    expect(body().text()).toContain('confirm the exact command');
    expect(body().text()).not.toContain('Secret values are stored');
  });

  it('says secrets are stored on disk when there are some', async () => {
    mountWizard();
    await settled();
    await pickCatalog('GitHub');
    await body().find('input[type="password"]').setValue('ghp_x');
    await click('Preview');
    expect(body().text()).toContain('Secret values are stored');
  });
});

describe('McpSetupWizard: editing', () => {
  it('opens at Configure with the stored values, asks the catalog for nothing, and shows no secret', async () => {
    mountWizard({ editing: server() });
    await settled();
    expect(sent.catalogCalls).toBe(0);
    expect(body().find('[role="dialog"]').attributes('aria-label')).toBe('Edit MCP Server');
    expect(body().find('[aria-current="step"]').text()).toContain('Configure');
    expect(body().text()).not.toContain('Select');
    expect((nameInput().element as HTMLInputElement).value).toBe('gh');
    expect((body().find('#mcp-command').element as HTMLInputElement).value).toBe('node');
    expect(argInputs().map((i) => (i.element as HTMLInputElement).value)).toEqual(['server.js', '--port 8080']);
    expect(body().findAll('input[type="password"]')).toHaveLength(0);
    expect(body().text().match(/Stored value kept/g)).toHaveLength(2);
    expect(body().findAll('button').some((b) => b.text() === 'Back')).toBe(false);
  });

  it('sends keep for every secret nobody touched (AC3)', async () => {
    mountWizard({ editing: server() });
    await settled();
    await click('Preview');
    expect(JSON.parse(previewText()).gh.env).toEqual({ TOKEN: '********', OTHER: '********' });
    await click('Save');
    expect(sent.update).toEqual([
      { originalName: 'gh', input: { name: 'gh', type: 'stdio', command: 'node', args: ['server.js', '--port 8080'], url: null, env: { TOKEN: { kind: 'keep' }, OTHER: { kind: 'keep' } }, headers: {} } },
    ]);
  });

  it('replaces one secret with a typed value, removes another, and can take either back', async () => {
    mountWizard({ editing: server() });
    await settled();
    await body().find('button[aria-label="Change the value of TOKEN"]').trigger('click');
    const field = body().find('input[aria-label="New value for TOKEN"]');
    expect((field.element as HTMLInputElement).value).toBe('');
    expect(field.attributes('type')).toBe('password');
    await field.setValue('  rotated-secret ');
    await body().find('button[aria-label="Remove OTHER"]').trigger('click');
    expect(body().text()).toContain('Will be removed');
    await click('Preview');
    expect(JSON.parse(previewText()).gh.env).toEqual({ TOKEN: '********' });
    expect(document.body.textContent).not.toContain('rotated-secret');
    await click('Save');
    expect(sent.update[0].input.env).toEqual({ TOKEN: { kind: 'set', value: 'rotated-secret' }, OTHER: { kind: 'remove' } });
  });

  it('goes back to keeping the stored value when a change is abandoned, and when a removal is undone', async () => {
    mountWizard({ editing: server() });
    await settled();
    await body().find('button[aria-label="Change the value of TOKEN"]').trigger('click');
    await body().find('button[aria-label="Keep the stored value of TOKEN"]').trigger('click');
    await body().find('button[aria-label="Remove OTHER"]').trigger('click');
    await body().find('button[aria-label="Undo removing OTHER"]').trigger('click');
    await click('Preview');
    await click('Save');
    expect(sent.update[0].input.env).toEqual({ TOKEN: { kind: 'keep' }, OTHER: { kind: 'keep' } });
  });

  it('refuses to set a changed secret to nothing', async () => {
    mountWizard({ editing: server() });
    await settled();
    await body().find('button[aria-label="Change the value of TOKEN"]').trigger('click');
    expect(button('Preview').attributes('disabled')).toBeDefined();
  });

  it('renames in place: the new name is sent with the original name to find the entry', async () => {
    mountWizard({ editing: server(), existingNames: ['gh', 'other'] });
    await settled();
    await nameInput().setValue('gh-renamed');
    await click('Preview');
    await click('Save');
    expect(sent.update[0].originalName).toBe('gh');
    expect(sent.update[0].input.name).toBe('gh-renamed');
  });

  it('lets the edited server keep its own name but not take another\'s', async () => {
    mountWizard({ editing: server(), existingNames: ['gh', 'other'] });
    await settled();
    expect(button('Preview').attributes('disabled')).toBeUndefined();
    await nameInput().setValue('other');
    expect(button('Preview').attributes('disabled')).toBeDefined();
  });

  it('keeps the transport, url and headers of an http server (what the Swift update dropped)', async () => {
    mountWizard({ editing: server({ type: 'http', command: null, args: [], url: 'https://x.example/mcp', envKeys: [], headerKeys: ['Authorization'] }) });
    await settled();
    expect((body().find('#mcp-url').element as HTMLInputElement).value).toBe('https://x.example/mcp');
    await click('Preview');
    await click('Save');
    expect(sent.update[0].input).toEqual({ name: 'gh', type: 'http', command: null, args: [], url: 'https://x.example/mcp', env: {}, headers: { Authorization: { kind: 'keep' } } });
  });

  it('says what it did and closes', async () => {
    const { onClose, toasts } = mountWizard({ editing: server() });
    await settled();
    await click('Preview');
    await click('Save');
    expect(toasts.current).toMatchObject({ message: 'Saved gh' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('says so, without claiming a save, when nothing changed on disk', async () => {
    const { toasts } = mountWizard({ editing: server(), outcome: 'unchanged' });
    await settled();
    await click('Preview');
    await click('Save');
    expect(toasts.current).toMatchObject({ message: 'No changes to gh', style: 'info' });
  });
});

describe('McpSetupWizard: keyboard (spec §6.2)', () => {
  const backdrop = () => body().find('.fixed.inset-0');

  it('is a labelled modal dialog', async () => {
    mountWizard();
    await settled();
    const dialog = body().find('[role="dialog"]');
    expect([dialog.attributes('aria-modal'), dialog.attributes('aria-label')]).toEqual(['true', 'Add MCP Server']);
  });

  it('Esc cancels, from any step', async () => {
    const { onClose } = mountWizard();
    await settled();
    await backdrop().trigger('keydown', { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Ctrl+. cancels on Windows/Linux and Cmd+. alone does not', async () => {
    const { onClose } = mountWizard({ os: 'windows' });
    await settled();
    await backdrop().trigger('keydown', { key: '.', metaKey: true });
    expect(onClose).not.toHaveBeenCalled();
    await backdrop().trigger('keydown', { key: '.', ctrlKey: true });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Return in the form goes to the preview while valid, and does not save', async () => {
    mountWizard({ editing: server() });
    await settled();
    await nameInput().trigger('keydown', { key: 'Enter' });
    await settled();
    expect(body().find('pre').exists()).toBe(true);
    expect(sent.update).toHaveLength(0);
  });

  it('Return in the form does nothing while it is invalid', async () => {
    mountWizard();
    await settled();
    await pickCustom();
    await nameInput().trigger('keydown', { key: 'Enter' });
    await settled();
    expect(body().find('pre').exists()).toBe(false);
  });

  it('Return in the preview saves once, even held down', async () => {
    mountWizard({ editing: server() });
    await settled();
    await click('Preview');
    void backdrop().trigger('keydown', { key: 'Enter' });
    void backdrop().trigger('keydown', { key: 'Enter' });
    await settled();
    expect(sent.update).toHaveLength(1);
  });

  it('Return on a focused button is the button\'s own, not the default action', async () => {
    mountWizard({ editing: server() });
    await settled();
    const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
    button('Cancel').element.dispatchEvent(event);
    await settled();
    expect(event.defaultPrevented).toBe(false);
    expect(body().find('pre').exists()).toBe(false);
  });

  describe('keys the app-level sidebar navigation would also act on stay in the dialog', () => {
    let seen: string[] = [];
    const spy = (event: KeyboardEvent) => seen.push(event.key);
    beforeEach(() => {
      seen = [];
      document.addEventListener('keydown', spy);
    });
    afterEach(() => document.removeEventListener('keydown', spy));

    it.each([['Enter'], ['Escape'], ['ArrowDown'], ['ArrowUp']])('%s does not bubble out', async (pressed) => {
      mountWizard({ editing: server() });
      await settled();
      await nameInput().trigger('keydown', { key: pressed });
      expect(seen).toEqual([]);
    });
  });
});

describe('McpSetupWizard: dismissal', () => {
  it('Cancel closes, and a backdrop click closes but a click inside does not', async () => {
    const { onClose } = mountWizard();
    await settled();
    await body().find('[role="dialog"]').trigger('click');
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('click');
    await button('Cancel').trigger('click');
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
