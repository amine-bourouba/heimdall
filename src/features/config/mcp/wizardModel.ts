// The logic of the MCP setup wizard (M3-48), kept out of the component so what is sent, what the
// preview shows and what counts as valid can be tested without mounting it.
//
// Secrets (ADR 0010 rule 8): an existing env var or header value never reaches the webview, so an
// existing row is only ever kept, replaced (`set`, with a value typed in this session) or removed.
// Nothing here is pre-filled from the stored server, and the preview shows every env and header value
// as the same fixed mask whatever the catalog says about the key (spec §10 item 2: Swift masked only
// catalog keys marked sensitive, so a custom server's secrets were printed in full).
import type { McpCatalogCategory, McpCatalogEntry, McpEnvKey, McpServer, McpServerInput, SecretInput } from '@/ipc/bindings';

export type Transport = 'stdio' | 'http' | 'sse';

export const TRANSPORTS: { value: Transport; label: string }[] = [
  { value: 'stdio', label: 'Command (stdio)' },
  { value: 'http', label: 'HTTP' },
  { value: 'sse', label: 'SSE' },
];

/** Every env and header value in the preview. Fixed: no prefix, no length, nothing that depends on the value. */
export const MASK = '********';

export const CATEGORY_ORDER: McpCatalogCategory[] = ['developer', 'productivity', 'data', 'filesystem'];
export const CATEGORY_LABELS: Record<McpCatalogCategory, string> = {
  developer: 'Developer Tools',
  productivity: 'Productivity',
  data: 'Data & Search',
  filesystem: 'Filesystem',
};

/** One env var or HTTP header. `existing` rows came from the stored server and have no value here. */
export interface SecretRow {
  /** Stable for `v-for`: the key is editable, so it cannot identify the row. */
  id: number;
  key: string;
  /** `keep`: leave the stored value alone. `set`: replace it with `value`. `remove`: delete it. */
  mode: 'keep' | 'set' | 'remove';
  value: string;
  existing: boolean;
  /** Catalog metadata (label, description, required, sensitive) when the key comes from the catalog. */
  meta: McpEnvKey | null;
}

export interface WizardModel {
  /** The catalog entry's id, or `null` for a custom server. */
  catalogId: string | null;
  /** The name being edited, or `null` when adding. */
  originalName: string | null;
  name: string;
  transport: Transport;
  command: string;
  /** An ordered list, never one string split on spaces (spec §10 item 1). */
  args: string[];
  url: string;
  env: SecretRow[];
  headers: SecretRow[];
}

let nextRowId = 1;

export function newRow(): SecretRow {
  return { id: nextRowId++, key: '', mode: 'set', value: '', existing: false, meta: null };
}

function existingRow(key: string): SecretRow {
  return { id: nextRowId++, key, mode: 'keep', value: '', existing: true, meta: null };
}

export function blankCustom(): WizardModel {
  return { catalogId: null, originalName: null, name: '', transport: 'stdio', command: '', args: [], url: '', env: [], headers: [] };
}

export function fromCatalog(entry: McpCatalogEntry): WizardModel {
  return {
    ...blankCustom(),
    catalogId: entry.id,
    name: entry.id,
    command: entry.command,
    args: [...entry.args],
    env: entry.envKeys.map((meta) => ({ id: nextRowId++, key: meta.id, mode: 'set', value: '', existing: false, meta })),
  };
}

/**
 * Editing always uses the custom form, even for a server that came from the catalog: the command, the
 * arguments (the filesystem server's directories are arguments) and each env var are then edited exactly
 * as stored, which a catalog re-derivation of the arguments could not guarantee.
 */
export function fromServer(server: McpServer): WizardModel {
  const known = server.type === 'stdio' || server.type === 'http' || server.type === 'sse';
  const transport: Transport = known ? (server.type as Transport) : server.url ? 'http' : 'stdio';
  return {
    catalogId: null,
    originalName: server.rawName,
    name: server.rawName,
    transport,
    command: server.command ?? '',
    args: [...server.args],
    url: server.url ?? '',
    env: server.envKeys.map(existingRow),
    headers: server.headerKeys.map(existingRow),
  };
}

// --- Catalog ------------------------------------------------------------------------------------

/** Plain substring match on name, description and category (MCPServerCatalog.swift:343-351). */
export function searchCatalog(entries: McpCatalogEntry[], query: string): McpCatalogEntry[] {
  const q = query.trim().toLowerCase();
  if (q === '') return entries;
  return entries.filter((e) => `${e.name} ${e.description} ${CATEGORY_LABELS[e.category]}`.toLowerCase().includes(q));
}

export function groupCatalog(entries: McpCatalogEntry[]): { category: McpCatalogCategory; label: string; entries: McpCatalogEntry[] }[] {
  return CATEGORY_ORDER.map((category) => ({ category, label: CATEGORY_LABELS[category], entries: entries.filter((e) => e.category === category) })).filter((g) => g.entries.length > 0);
}

// --- Arguments and env, with the two catalog special cases ---------------------------------------

const FILESYSTEM_ID = 'filesystem';
const ALLOWED_DIRS = 'ALLOWED_DIRS';

function splitDirs(value: string): string[] {
  return value
    .split(',')
    .map((d) => d.trim())
    .filter((d) => d !== '');
}

/** The filesystem server takes its directories as arguments, not as an env var (MCPSetupWizardView.swift:61-92). */
function isDirsRow(model: WizardModel, row: SecretRow): boolean {
  return model.catalogId === FILESYSTEM_ID && row.meta !== null && row.key === ALLOWED_DIRS;
}

/** A row nobody filled in: a new, key-less, value-less row (a spare line), or an optional catalog variable left empty. */
function isIgnored(row: SecretRow): boolean {
  if (row.existing) return false;
  if (row.meta) return !row.meta.isRequired && row.value.trim() === '';
  return row.key.trim() === '' && row.value === '';
}

function sentRows(model: WizardModel, rows: SecretRow[]): SecretRow[] {
  return rows.filter((row) => !isDirsRow(model, row) && !isIgnored(row) && !(row.mode === 'remove' && !row.existing));
}

function secretMap(rows: SecretRow[]): Record<string, SecretInput> {
  const out: Record<string, SecretInput> = {};
  for (const row of rows) {
    const key = row.key.trim();
    // A pasted token often carries a trailing newline or space that would break authentication.
    out[key] = row.mode === 'keep' ? { kind: 'keep' } : row.mode === 'remove' ? { kind: 'remove' } : { kind: 'set', value: row.value.trim() };
  }
  return out;
}

function resolvedArgs(model: WizardModel, entry: McpCatalogEntry | null): string[] {
  const args = model.args.filter((a) => a.trim() !== '');
  if (!entry || entry.id !== FILESYSTEM_ID) return args;
  const dirs = model.env.find((row) => isDirsRow(model, row));
  return [...args, ...splitDirs(dirs?.value ?? '')];
}

/** What is sent to `add_mcp_server` / `update_mcp_server`. */
export function toInput(model: WizardModel, entry: McpCatalogEntry | null): McpServerInput {
  const name = model.name.trim();
  if (model.transport === 'stdio') {
    return { name, type: 'stdio', command: model.command.trim(), args: resolvedArgs(model, entry), url: null, env: secretMap(sentRows(model, model.env)), headers: {} };
  }
  return { name, type: model.transport, command: null, args: [], url: model.url.trim(), env: {}, headers: secretMap(sentRows(model, model.headers)) };
}

// --- Preview ------------------------------------------------------------------------------------

function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((k) => [k, sortKeys((value as Record<string, unknown>)[k])]),
    );
  }
  return value;
}

function masked(map: Record<string, SecretInput>): Record<string, string> {
  return Object.fromEntries(Object.entries(map).filter(([, s]) => s.kind !== 'remove').map(([k]) => [k, MASK]));
}

/**
 * `{ "<name>": { ... } }` as it will be written, with sorted keys and **every env and header value
 * masked the same way**. It mirrors what the Rust writer stores: `type`, then `command` and `args` (and
 * `env` if any) for stdio, or `url` (and `headers` if any) for http and sse.
 */
export function previewJson(model: WizardModel, entry: McpCatalogEntry | null): string {
  const input = toInput(model, entry);
  const body: Record<string, unknown> = { type: input.type };
  if (input.type === 'stdio') {
    body.command = input.command;
    body.args = input.args;
    const env = masked(input.env);
    if (Object.keys(env).length > 0) body.env = env;
  } else {
    body.url = input.url;
    const headers = masked(input.headers);
    if (Object.keys(headers).length > 0) body.headers = headers;
  }
  return JSON.stringify({ [input.name]: sortKeys(body) }, null, 2);
}

// --- Validation ---------------------------------------------------------------------------------

export interface Problems {
  name?: string;
  command?: string;
  url?: string;
  env?: string;
  headers?: string;
}

export function hasProblems(problems: Problems): boolean {
  return Object.keys(problems).length > 0;
}

function rowProblem(model: WizardModel, rows: SecretRow[], kind: 'env' | 'header'): string | undefined {
  const filled = rows.filter((row) => row.mode !== 'remove' && !isIgnored(row));
  // The directories row is required like any other, and must name at least one directory: it becomes arguments.
  const empty = (row: SecretRow) => (isDirsRow(model, row) ? splitDirs(row.value).length === 0 : row.value.trim() === '');
  const missing = filled.filter((row) => row.meta?.isRequired && empty(row)).map((row) => row.meta!.label || row.key);
  const live = filled.filter((row) => !isDirsRow(model, row));
  if (missing.length > 0) return `Fill in the required value${missing.length > 1 ? 's' : ''}: ${missing.join(', ')}.`;
  const seen = new Set<string>();
  for (const row of live) {
    const key = row.key.trim();
    if (key === '') return `Give every ${kind === 'env' ? 'variable' : 'header'} a name, or remove the empty line.`;
    if (/\s/.test(key)) return `"${key}" cannot contain spaces.`;
    if (kind === 'env' && key.includes('=')) return `"${key}" cannot contain "=".`;
    if (kind === 'header' && key.includes(':')) return `"${key}" cannot contain ":".`;
    const folded = kind === 'header' ? key.toLowerCase() : key;
    if (seen.has(folded)) return `"${key}" appears twice.`;
    seen.add(folded);
    if (row.mode === 'set' && row.value.trim() === '') return `Enter a value for "${key}", keep the stored one, or remove it.`;
  }
  return undefined;
}

function isHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === 'http:' || url.protocol === 'https:';
  } catch {
    return false;
  }
}

/** `takenNames`: the keys already in `mcpServers` (the one being edited may keep its own name). */
export function validate(model: WizardModel, takenNames: string[]): Problems {
  const problems: Problems = {};
  const name = model.name.trim();
  if (name === '') problems.name = 'Enter a name.';
  else if (name !== model.originalName && takenNames.includes(name)) problems.name = `A server named "${name}" already exists.`;
  if (model.transport === 'stdio') {
    if (model.catalogId === null && model.command.trim() === '') problems.command = 'Enter the command to run.';
    const env = rowProblem(model, model.env, 'env');
    if (env) problems.env = env;
  } else {
    if (model.url.trim() === '') problems.url = 'Enter the server URL.';
    else if (!isHttpUrl(model.url.trim())) problems.url = 'Enter a valid http:// or https:// URL.';
    const headers = rowProblem(model, model.headers, 'header');
    if (headers) problems.headers = headers;
  }
  return problems;
}
