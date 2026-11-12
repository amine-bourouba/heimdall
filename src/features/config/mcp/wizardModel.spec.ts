// M3-48's three acceptance criteria live here, as logic: arguments are an ordered list and never split
// on spaces; the preview masks every env and header value completely for catalog and custom servers;
// editing sends `keep` for a secret nobody touched.
import { describe, expect, it } from 'vitest';
import type { McpCatalogEntry, McpEnvKey, McpServer } from '@/ipc/bindings';
import {
  MASK,
  blankCustom,
  fromCatalog,
  fromServer,
  groupCatalog,
  hasProblems,
  newRow,
  previewJson,
  searchCatalog,
  toInput,
  validate,
  type WizardModel,
} from './wizardModel';

function envKey(id: string, overrides: Partial<McpEnvKey> = {}): McpEnvKey {
  return { id, label: id, description: '', placeholder: '', isRequired: true, isSensitive: false, ...overrides };
}

function entry(id: string, overrides: Partial<McpCatalogEntry> = {}): McpCatalogEntry {
  return { id, name: id, description: `The ${id} server`, icon: 'hammer', category: 'developer', command: 'npx', args: ['-y', `@scope/${id}`], envKeys: [], ...overrides };
}

const GITHUB = entry('github', { envKeys: [envKey('GITHUB_PERSONAL_ACCESS_TOKEN', { isSensitive: true })] });
const FILESYSTEM = entry('filesystem', { category: 'filesystem', envKeys: [envKey('ALLOWED_DIRS')] });
const SQLITE = entry('sqlite', { category: 'data', envKeys: [envKey('SQLITE_DB_PATH')] });
const SENTRY = entry('sentry', { envKeys: [envKey('SENTRY_AUTH_TOKEN', { isSensitive: true }), envKey('SENTRY_ORG', { isRequired: false })] });

function server(overrides: Partial<McpServer> = {}): McpServer {
  return { id: 'user:gh', name: 'Gh', rawName: 'gh', scope: 'global', source: 'user', type: 'stdio', command: 'node', args: ['server.js'], url: null, envKeys: [], headerKeys: [], tools: [], isWildcard: false, ...overrides };
}

function customStdio(overrides: Partial<WizardModel> = {}): WizardModel {
  return { ...blankCustom(), name: 'tool', command: 'node', ...overrides };
}

function setRow(key: string, value: string) {
  return { ...newRow(), key, value };
}

describe('arguments are an ordered list, never split on spaces (spec §10 item 1)', () => {
  it('keeps an argument that contains spaces, quotes or JSON as one entry, in order', () => {
    const model = customStdio({ args: ['--config', '/Users/a b/my config.json', '{"a": 1, "b": [2, 3]}', '--name=hello world'] });
    expect(toInput(model, null).args).toEqual(['--config', '/Users/a b/my config.json', '{"a": 1, "b": [2, 3]}', '--name=hello world']);
  });

  it('drops only entirely blank entries and leaves everything else exactly as typed', () => {
    const model = customStdio({ args: ['-y', '', '   ', ' spaced ', '@scope/server'] });
    expect(toInput(model, null).args).toEqual(['-y', ' spaced ', '@scope/server']);
  });

  it('puts the command and the arguments in different fields', () => {
    const input = toInput(customStdio({ command: ' npx ', args: ['-y', 'pkg'] }), null);
    expect([input.command, input.args]).toEqual(['npx', ['-y', 'pkg']]);
  });
});

describe('the preview masks every env and header value, whatever the catalog says (spec §10 item 2)', () => {
  const SECRETS = ['sk-live-ABCD1234', 'abcd', 'p@ss w0rd', 'plain-value'];

  function expectNoneOf(text: string, values: string[]) {
    for (const value of values) expect(text, `"${value}" must not appear`).not.toContain(value);
  }

  it('masks a custom stdio server\'s env, including a short value and a value that looks harmless', () => {
    const model = customStdio({ env: [setRow('API_KEY', 'sk-live-ABCD1234'), setRow('PIN', 'abcd'), setRow('MODE', 'plain-value')] });
    const preview = previewJson(model, null);
    expectNoneOf(preview, SECRETS);
    expect(JSON.parse(preview).tool.env).toEqual({ API_KEY: MASK, PIN: MASK, MODE: MASK });
  });

  it('masks a custom http server\'s headers', () => {
    const model: WizardModel = { ...blankCustom(), name: 'remote', transport: 'http', url: 'https://x.example/mcp', headers: [setRow('Authorization', 'Bearer p@ss w0rd')] };
    const preview = previewJson(model, null);
    expectNoneOf(preview, ['p@ss w0rd', 'Bearer']);
    expect(JSON.parse(preview).remote).toEqual({ type: 'http', url: 'https://x.example/mcp', headers: { Authorization: MASK } });
  });

  it('masks a catalog server\'s sensitive and non-sensitive variables alike', () => {
    const model = fromCatalog(SENTRY);
    model.env[0].value = 'sk-live-ABCD1234';
    model.env[1].value = 'plain-value'; // SENTRY_ORG is not sensitive: still masked
    const preview = previewJson(model, SENTRY);
    expectNoneOf(preview, ['sk-live-ABCD1234', 'plain-value', 'sk-l']);
    expect(JSON.parse(preview).sentry.env).toEqual({ SENTRY_AUTH_TOKEN: MASK, SENTRY_ORG: MASK });
  });

  it('uses one fixed mask: nothing about the value (its prefix, its length) shows through', () => {
    const short = previewJson(customStdio({ env: [setRow('K', 'x')] }), null);
    const long = previewJson(customStdio({ env: [setRow('K', 'x'.repeat(80))] }), null);
    expect(short).toBe(long);
  });

  it('masks a kept (stored) value, which the webview never held, and omits a removed one', () => {
    const edited = fromServer(server({ envKeys: ['KEPT', 'GONE'] }));
    edited.env[1].mode = 'remove';
    expect(JSON.parse(previewJson(edited, null)).gh.env).toEqual({ KEPT: MASK });
  });

  it('shows the structure as it will be written: sorted keys, type first in the object it describes', () => {
    const preview = previewJson(customStdio({ args: ['b', 'a'], env: [setRow('Z', '1'), setRow('A', '2')] }), null);
    expect(preview.indexOf('"A"')).toBeLessThan(preview.indexOf('"Z"'));
    expect(JSON.parse(preview).tool).toEqual({ type: 'stdio', command: 'node', args: ['b', 'a'], env: { A: MASK, Z: MASK } });
  });

  it('writes no env or headers key at all when there are none', () => {
    expect(JSON.parse(previewJson(customStdio(), null)).tool).toEqual({ type: 'stdio', command: 'node', args: [] });
  });
});

describe('editing: untouched secrets are sent as keep (never as their value)', () => {
  it('opens every stored env var and header as an untouched row with no value in the webview', () => {
    const model = fromServer(server({ envKeys: ['A', 'B'], headerKeys: ['Authorization'] }));
    expect([...model.env, ...model.headers].map((r) => [r.mode, r.value, r.existing])).toEqual([
      ['keep', '', true],
      ['keep', '', true],
      ['keep', '', true],
    ]);
  });

  it('sends keep for an untouched row, set for a changed one, remove for a removed one, set for a new one', () => {
    const model = fromServer(server({ envKeys: ['UNTOUCHED', 'CHANGED', 'REMOVED'] }));
    model.env[1].mode = 'set';
    model.env[1].value = '  new-secret \n';
    model.env[2].mode = 'remove';
    model.env.push(setRow('ADDED', 'added-secret'));
    expect(toInput(model, null).env).toEqual({
      UNTOUCHED: { kind: 'keep' },
      CHANGED: { kind: 'set', value: 'new-secret' },
      REMOVED: { kind: 'remove' },
      ADDED: { kind: 'set', value: 'added-secret' },
    });
  });

  it('sends nothing for a row that was added and then removed before saving', () => {
    const model = customStdio({ env: [{ ...setRow('TEMP', 'v'), mode: 'remove' }] });
    expect(toInput(model, null).env).toEqual({});
  });

  it('keeps the transport, url and headers of an http server (the Swift update dropped them)', () => {
    const model = fromServer(server({ type: 'http', command: null, args: [], url: 'https://x.example/mcp', headerKeys: ['Authorization'] }));
    expect(toInput(model, null)).toEqual({
      name: 'gh',
      type: 'http',
      command: null,
      args: [],
      url: 'https://x.example/mcp',
      env: {},
      headers: { Authorization: { kind: 'keep' } },
    });
  });

  it('opens a server with no type as http when it has a url, else as stdio', () => {
    expect(fromServer(server({ type: null, url: 'https://x', command: null })).transport).toBe('http');
    expect(fromServer(server({ type: null })).transport).toBe('stdio');
    expect(fromServer(server({ type: 'sse', command: null, url: 'https://x' })).transport).toBe('sse');
  });

  it('edits through the custom form, whatever the server was created from', () => {
    const model = fromServer(server({ rawName: 'filesystem', args: ['-y', '@modelcontextprotocol/server-filesystem', '/home/a'] }));
    expect(model.catalogId).toBeNull();
    expect(toInput(model, null).args).toEqual(['-y', '@modelcontextprotocol/server-filesystem', '/home/a']);
  });
});

describe('the catalog path', () => {
  it('starts a catalog server named after it, with its command and arguments, and empty variables', () => {
    const model = fromCatalog(GITHUB);
    expect([model.name, model.command, model.args, model.catalogId]).toEqual(['github', 'npx', ['-y', '@scope/github'], 'github']);
    expect(model.env.map((r) => [r.key, r.value, r.existing])).toEqual([['GITHUB_PERSONAL_ACCESS_TOKEN', '', false]]);
  });

  it('sends the catalog command and arguments, and its variables as set values', () => {
    const model = fromCatalog(GITHUB);
    model.env[0].value = 'ghp_secret';
    expect(toInput(model, GITHUB)).toEqual({
      name: 'github',
      type: 'stdio',
      command: 'npx',
      args: ['-y', '@scope/github'],
      url: null,
      env: { GITHUB_PERSONAL_ACCESS_TOKEN: { kind: 'set', value: 'ghp_secret' } },
      headers: {},
    });
  });

  it('requires a required variable and skips an optional one left empty', () => {
    const model = fromCatalog(SENTRY);
    expect(validate(model, []).env).toContain('SENTRY_AUTH_TOKEN');
    model.env[0].value = 'tok';
    expect(hasProblems(validate(model, []))).toBe(false);
    expect(Object.keys(toInput(model, SENTRY).env)).toEqual(['SENTRY_AUTH_TOKEN']);
  });

  it('turns the filesystem server\'s comma-separated directories into trimmed arguments, not an env var', () => {
    const model = fromCatalog(FILESYSTEM);
    model.env[0].value = ' /home/a , /srv/data ,, ';
    const input = toInput(model, FILESYSTEM);
    expect(input.args).toEqual(['-y', '@scope/filesystem', '/home/a', '/srv/data']);
    expect(input.env).toEqual({});
  });

  it('requires at least one filesystem directory', () => {
    const model = fromCatalog(FILESYSTEM);
    expect(validate(model, []).env).toContain('ALLOWED_DIRS');
    model.env[0].value = ' , ,';
    expect(validate(model, []).env).toContain('ALLOWED_DIRS');
    model.env[0].value = '/home/a';
    expect(hasProblems(validate(model, []))).toBe(false);
  });

  it('writes the SQLite database path as a variable instead of silently dropping it (the Swift wizard collected it and wrote nothing)', () => {
    const model = fromCatalog(SQLITE);
    model.env[0].value = '/data/app.db';
    expect(toInput(model, SQLITE).env).toEqual({ SQLITE_DB_PATH: { kind: 'set', value: '/data/app.db' } });
  });
});

describe('http and sse servers', () => {
  const remote = (overrides: Partial<WizardModel> = {}): WizardModel => ({ ...blankCustom(), name: 'remote', transport: 'http', url: ' https://x.example/mcp ', ...overrides });

  it('sends a trimmed url and headers, and no command, arguments or env', () => {
    const input = toInput(remote({ headers: [setRow('Authorization', 'Bearer t')], env: [setRow('IGNORED', 'v')] }), null);
    expect(input).toEqual({ name: 'remote', type: 'http', command: null, args: [], url: 'https://x.example/mcp', env: {}, headers: { Authorization: { kind: 'set', value: 'Bearer t' } } });
  });

  it('accepts only http and https URLs', () => {
    expect(validate(remote(), []).url).toBeUndefined();
    expect(validate(remote({ url: 'ftp://x.example' }), []).url).toBeDefined();
    expect(validate(remote({ url: 'not a url' }), []).url).toBeDefined();
    expect(validate(remote({ url: '  ' }), []).url).toBeDefined();
  });

  it('does not ask an http server for a command', () => {
    expect(validate(remote({ command: '' }), []).command).toBeUndefined();
  });
});

describe('validation', () => {
  it('needs a name, and a command for a custom stdio server', () => {
    expect(validate(blankCustom(), [])).toMatchObject({ name: expect.any(String), command: expect.any(String) });
    expect(hasProblems(validate(customStdio(), []))).toBe(false);
  });

  it('refuses a name that is already taken, but lets an edited server keep its own', () => {
    expect(validate(customStdio({ name: 'gh' }), ['gh']).name).toContain('already exists');
    const editing = fromServer(server());
    expect(validate(editing, ['gh', 'other']).name).toBeUndefined();
    editing.name = 'other';
    expect(validate(editing, ['gh', 'other']).name).toContain('already exists');
  });

  it('checks each env var: a name, no spaces or "=", no repeats, and a value to set', () => {
    expect(validate(customStdio({ env: [setRow('', 'v')] }), []).env).toContain('name');
    expect(validate(customStdio({ env: [setRow('A B', 'v')] }), []).env).toContain('spaces');
    expect(validate(customStdio({ env: [setRow('A=B', 'v')] }), []).env).toContain('=');
    expect(validate(customStdio({ env: [setRow('A', '1'), setRow('A', '2')] }), []).env).toContain('twice');
    expect(validate(customStdio({ env: [setRow('A', '')] }), []).env).toContain('value');
    expect(validate(customStdio({ env: [setRow('A', 'v')] }), []).env).toBeUndefined();
  });

  it('ignores a spare empty line, and a removed row', () => {
    expect(validate(customStdio({ env: [newRow(), { ...setRow('GONE', ''), mode: 'remove' }] }), []).env).toBeUndefined();
  });

  it('treats header names case-insensitively and refuses ":"', () => {
    const remote: WizardModel = { ...blankCustom(), name: 'r', transport: 'http', url: 'https://x' };
    expect(validate({ ...remote, headers: [setRow('Authorization', 'a'), setRow('authorization', 'b')] }, []).headers).toContain('twice');
    expect(validate({ ...remote, headers: [setRow('X:Y', 'a')] }, []).headers).toContain(':');
  });

  it('lets an existing row be kept without a value', () => {
    expect(hasProblems(validate(fromServer(server({ envKeys: ['A'] })), ['gh']))).toBe(false);
  });
});

describe('the catalog list', () => {
  const all = [entry('b-tool', { category: 'data', name: 'Beta' }), entry('a-tool', { name: 'Alpha', description: 'Reads GitHub issues' }), entry('fs', { category: 'filesystem', name: 'Files' })];

  it('searches name, description and category by plain substring, ignoring case', () => {
    expect(searchCatalog(all, 'github').map((e) => e.id)).toEqual(['a-tool']);
    expect(searchCatalog(all, 'DATA & SEARCH').map((e) => e.id)).toEqual(['b-tool']);
    expect(searchCatalog(all, '  ').length).toBe(3);
    expect(searchCatalog(all, 'zzz')).toEqual([]);
  });

  it('groups by category in the fixed order, leaving out an empty one', () => {
    expect(groupCatalog(all).map((g) => [g.label, g.entries.map((e) => e.id)])).toEqual([
      ['Developer Tools', ['a-tool']],
      ['Data & Search', ['b-tool']],
      ['Filesystem', ['fs']],
    ]);
  });
});
