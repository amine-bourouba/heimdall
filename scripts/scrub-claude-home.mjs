#!/usr/bin/env node
// Copy a real Claude Code home into a fixture folder with personal data removed (task M0-13).
//
//   node scripts/scrub-claude-home.mjs --home <~/.claude> [--config <~/.claude.json>] --out <dir>
//                                      [--user <name>] [--max-bytes 5000000]
//
// Output: <out>/claude-home/** and <out>/claude.json. Prints a JSON summary (file counts, line endings).
//
// Rules:
// - Only paths Heimdall reads are copied (allowlist below); anything named *credentials* is never copied.
// - The user's home and name are replaced by `fixture` in every string, object key and folder name,
//   including hyphen-encoded project folders (C--Users-<name>-..., -Users-<name>-..., -home-<name>-...).
// - Secrets (strings under keys matching SECRET_KEY) become "***scrubbed***".
// - Object keys that identify an account or repository are replaced too: UUID keys (account and
//   organisation caches in ~/.claude.json) become stable fake UUIDs, `githubRepoPaths` keys become
//   `fixture/repo-<n>`. The same fake UUIDs replace the ids in `skills/synced/<id>` folder names.
// - Structural fields (STRUCTURAL) keep their value after path scrubbing; every other string becomes
//   same-length filler that keeps whitespace, so line counts and diff shapes survive.
// - JSONL line endings are preserved byte for byte; unparsable lines stay unparsable.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { pathToFileURL } from 'node:url';

export const SECRET_KEY = /token|key|secret|password|authorization|env|headers/i;
const STRUCTURAL = new Set([
  'type', 'role', 'uuid', 'parentUuid', 'leafUuid', 'sessionId', 'session_id', 'id', 'tool_use_id', 'name', 'model',
  'timestamp', 'created', 'modified', 'version', 'userType', 'stop_reason', 'stop_sequence', 'subtype', 'level',
  'agentId', 'agentType', 'status', 'backupFileName', 'backupTime', 'cwd', 'projectPath', 'originalPath', 'project',
  'trackingPath', 'file_path', 'filePath', 'path', 'notebook_path', 'messageId', 'requestId', 'outcome',
  'claude_helpfulness', 'session_type', 'lastComputedDate', 'date', 'firstSessionDate', 'installedAt',
  'color', 'memory', 'matcher', 'isSidechain', 'isMeta', 'isCompactSummary', 'entrypoint',
]);
// Top-level entries under ~/.claude that Heimdall reads (docs/spec/claude-home-inventory.md).
const ALLOW = ['projects', 'history.jsonl', 'todos', 'debug', 'file-history', 'usage-data', 'stats-cache.json',
  'commands', 'skills', 'plans', 'output-styles', 'agents', 'settings.json', 'plugins/installed_plugins.json',
  'mcp-needs-auth-cache.json'];
const FILLER = 'loremipsumdolorsitametconsecteturadipiscingelitseddoeiusmodtemporincididuntutlabore';

function escapeRe(s) { return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); }

// Folder names that identify nothing and stay as they are inside project paths.
const GENERIC_SEGMENTS = new Set(['Desktop', 'Documents', 'Downloads', 'src', 'source', 'repos', 'repo', 'projects',
  'Projects', 'code', 'Code', 'dev', 'work', 'tech', 'git', 'workspace', 'workspaces', 'github', 'GitHub', 'Sites', 'tmp']);
const HOME_RE = /(?:[A-Za-z]:[\\/]+Users|\/[a-z]\/Users|\/Users|\/home)[\\/]+fixture((?:[\\/]+[^\\/]+)*)/g;

// Claude Code names a project folder by replacing every non-alphanumeric character of its path with '-'.
export const encodeSegment = (s) => s.replace(/[^A-Za-z0-9]/g, '-');

// Pseudonyms for the path segments of project roots below the home folder (the names that identify work).
export function projectSegmentMap(projectRoots) {
  const segs = new Set();
  for (const root of projectRoots) {
    for (const m of root.matchAll(HOME_RE)) {
      for (const seg of m[1].split(/[\\/]+/)) if (seg && !GENERIC_SEGMENTS.has(seg)) segs.add(seg);
    }
  }
  return new Map([...segs].sort().map((s, i) => [s, `proj-${i + 1}`]));
}

export function makePathScrubber(user, segMap = new Map()) {
  const u = escapeRe(user);
  const rules = [
    [new RegExp(`([A-Za-z]:[\\\\/]+Users[\\\\/]+)${u}(?=[\\\\/"]|$)`, 'gi'), '$1fixture'],
    [new RegExp(`(/[a-z]/Users/)${u}(?=/|$)`, 'gi'), '$1fixture'],
    [new RegExp(`(/(?:home|Users)/)${u}(?=/|$)`, 'g'), '$1fixture'],
    [new RegExp(`([A-Za-z]--Users-)${u}(?=-|$)`, 'gi'), '$1fixture'],
    [new RegExp(`(-(?:home|Users)-)${u}(?=-|$)`, 'g'), '$1fixture'],
    [new RegExp(`\\b${u}\\b`, 'gi'), 'fixture'],
    [/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, 'fixture@example.com'],
  ];
  const segs = [...segMap.entries()].sort((a, b) => b[0].length - a[0].length);
  const segRules = segs.map(([seg, alias]) => [new RegExp(`(^|[\\\\/])${escapeRe(seg)}(?=[\\\\/]|$)`, 'g'), `$1${alias}`]);
  // Paths can also contain hyphen-encoded project folders (`...\.claude\projects\C--Users-fixture-app\...`).
  const scrubFolder = makeFolderScrubber(segMap);
  const encoded = /(?:[A-Za-z]--Users|-Users|-home)-fixture[A-Za-z0-9-]*/g;
  return (s) => segRules
    .reduce((acc, [re, rep]) => acc.replace(re, rep), rules.reduce((acc, [re, rep]) => acc.replace(re, rep), s))
    .replace(encoded, scrubFolder);
}

// Same pseudonyms applied to a hyphen-encoded project folder name (after the user name is replaced).
export function makeFolderScrubber(segMap) {
  const segs = [...segMap.entries()].map(([seg, alias]) => [encodeSegment(seg), alias]).sort((a, b) => b[0].length - a[0].length);
  return (name) => {
    let out = name;
    for (const [enc, alias] of segs) out = out.replace(new RegExp(`-${escapeRe(enc)}(?=-|$)`, 'g'), `-${alias}`);
    return out;
  };
}

// Absolute project roots recorded anywhere in the tree: transcript cwd, sessions-index paths, history, ~/.claude.json keys.
function collectProjectRoots(home, config, basicScrub) {
  const roots = new Set();
  const add = (p) => { if (typeof p === 'string') roots.add(basicScrub(p)); };
  const projects = path.join(home, 'projects');
  if (fs.existsSync(projects)) {
    for (const dir of fs.readdirSync(projects, { withFileTypes: true })) {
      if (!dir.isDirectory()) continue;
      const pdir = path.join(projects, dir.name);
      for (const f of fs.readdirSync(pdir)) {
        const file = path.join(pdir, f);
        if (f === 'sessions-index.json') {
          try {
            const idx = JSON.parse(fs.readFileSync(file, 'utf8'));
            add(idx.projectPath); add(idx.originalPath);
            for (const e of idx.entries || []) add(e.projectPath);
          } catch { /* unreadable index: nothing to collect */ }
        } else if (f.endsWith('.jsonl')) {
          for (const line of fs.readFileSync(file, 'utf8').split('\n')) {
            const m = line.match(/"cwd":"((?:[^"\\]|\\.)*)"/);
            if (m) { try { add(JSON.parse(`"${m[1]}"`)); } catch { /* skip */ } }
          }
        }
      }
    }
  }
  const history = path.join(home, 'history.jsonl');
  if (fs.existsSync(history)) {
    for (const line of fs.readFileSync(history, 'utf8').split('\n')) {
      try { add(JSON.parse(line).project); } catch { /* skip */ }
    }
  }
  if (config && fs.existsSync(config)) {
    try { for (const k of Object.keys(JSON.parse(fs.readFileSync(config, 'utf8')).projects || {})) add(k); } catch { /* skip */ }
  }
  return [...roots];
}

export function filler(s) {
  let i = 0;
  let out = '';
  for (const ch of s) out += /\s/.test(ch) ? ch : FILLER[i++ % FILLER.length];
  return out;
}

const UUID_KEY = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const UUID_ANY = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi;
const fakeUuids = new Map();
const fakeUuid = (id) => {
  if (!fakeUuids.has(id)) fakeUuids.set(id, `00000000-0000-4000-8000-${String(fakeUuids.size + 1).padStart(12, '0')}`);
  return fakeUuids.get(id);
};

function scrubKey(k, parentKey, index, scrubPath) {
  if (parentKey === 'githubRepoPaths') return `fixture/repo-${index + 1}`;
  return UUID_KEY.test(k) ? fakeUuid(k.toLowerCase()) : scrubPath(k);
}

function scrubValue(value, key, scrubPath, inSecret) {
  if (typeof value === 'string') {
    if (inSecret || (key && SECRET_KEY.test(key))) return '***scrubbed***';
    return key && STRUCTURAL.has(key) ? scrubPath(value) : filler(scrubPath(value));
  }
  if (Array.isArray(value)) return value.map((v) => scrubValue(v, key, scrubPath, inSecret));
  if (value && typeof value === 'object') {
    const secret = inSecret || Boolean(key && SECRET_KEY.test(key) && typeof value === 'object');
    const out = {};
    Object.entries(value).forEach(([k, v], i) => { out[scrubKey(k, key, i, scrubPath)] = scrubValue(v, k, scrubPath, secret); });
    return out;
  }
  return value;
}

export function scrubJsonl(text, scrubPath) {
  const parts = text.split(/(\r\n|\n)/); // keep terminators
  let bad = 0;
  for (let i = 0; i < parts.length; i += 2) {
    const line = parts[i];
    if (!line) continue;
    try {
      parts[i] = JSON.stringify(scrubValue(JSON.parse(line), null, scrubPath, false));
    } catch {
      bad++;
      parts[i] = filler(scrubPath(line)).replace(/^./, '{'); // still unparsable, same length
    }
  }
  return { text: parts.join(''), bad };
}

export function scrubJson(text, scrubPath) {
  const indent = (text.match(/^[{[]\r?\n([ \t]+)/) || [])[1] ?? '';
  const eol = text.includes('\r\n') ? '\r\n' : '\n';
  const body = JSON.stringify(scrubValue(JSON.parse(text), null, scrubPath, false), null, indent || undefined);
  const out = (indent ? body.replace(/\n/g, eol) : body) + (/\r?\n$/.test(text) ? eol : '');
  return out;
}

export function scrubMarkdown(text, scrubPath) {
  const m = text.match(/^---\r?\n([\s\S]*?)\r?\n---(\r?\n|$)/);
  if (!m) return filler(scrubPath(text));
  const front = m[1].split(/(\r?\n)/).map((line) => {
    const kv = line.match(/^([A-Za-z0-9_-]+):(\s*)(.*)$/);
    if (!kv) return /\r?\n/.test(line) ? line : filler(scrubPath(line));
    const keep = ['model', 'color', 'memory', 'tools', 'allowed-tools'].includes(kv[1]);
    return `${kv[1]}:${kv[2]}${keep ? scrubPath(kv[3]) : filler(scrubPath(kv[3]))}`;
  }).join('');
  const eol = m[0].includes('\r\n') ? '\r\n' : '\n';
  return `---${eol}${front}${eol}---${m[2]}${filler(scrubPath(text.slice(m[0].length)))}`;
}

export function scrubDebugLog(text, scrubPath) {
  return text.split(/(\r\n|\n)/).map((line) => {
    const m = line.match(/^(\d{4}-\d\d-\d\dT[^ ]+ \[[A-Z]+\] )(.*)$/);
    return m ? m[1] + filler(scrubPath(m[2])) : /^\r?\n$/.test(line) ? line : filler(scrubPath(line));
  }).join('');
}

function capJsonl(text, maxBytes) {
  if (Buffer.byteLength(text) <= maxBytes) return { text, capped: false };
  let out = '';
  for (const piece of text.match(/[^\n]*\n|[^\n]+$/g) || []) {
    if (Buffer.byteLength(out) + Buffer.byteLength(piece) > maxBytes) break;
    out += piece;
  }
  return { text: out, capped: true };
}

function allowed(rel) {
  const norm = rel.split(path.sep).join('/');
  if (/credentials/i.test(norm)) return false;
  return ALLOW.some((a) => norm === a || norm.startsWith(a + '/'));
}

export function scrubTree({ home, config, out, user, maxBytes = 5_000_000 }) {
  const segMap = projectSegmentMap(collectProjectRoots(home, config, makePathScrubber(user)));
  const scrubPath = makePathScrubber(user, segMap);
  const scrubFolder = makeFolderScrubber(segMap);
  const summary = { user: 'fixture', files: 0, pseudonymisedSegments: segMap.size, unmappedFolders: [], capped: [],
    skippedUnreadable: 0, unparsableLines: 0, jsonl: { crlf: 0, lf: 0 } };
  const destRoot = path.join(out, 'claude-home');
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const src = path.join(dir, entry.name);
      const rel = path.relative(home, src);
      if (entry.isDirectory()) {
        if (ALLOW.some((a) => a === rel.split(path.sep).join('/') || a.startsWith(rel.split(path.sep).join('/') + '/')) || allowed(rel)) walk(src);
        continue;
      }
      if (!entry.isFile() || !allowed(rel)) continue;
      const parts = rel.split(path.sep).map(scrubPath);
      if (parts[0] === 'projects' && parts.length > 2) parts[1] = scrubFolder(parts[1]);
      if (parts[0] === 'skills' && parts[1] === 'synced' && parts.length > 3) parts[2] = parts[2].replace(UUID_ANY, (u) => fakeUuid(u.toLowerCase()));
      const destRel = parts.join(path.sep);
      const dest = path.join(destRoot, destRel);
      let text;
      try { text = fs.readFileSync(src, 'utf8'); } catch { summary.skippedUnreadable++; continue; }
      let result;
      try {
        if (src.endsWith('.jsonl')) {
          const capped = capJsonl(text, maxBytes);
          if (capped.capped) summary.capped.push(destRel.split(path.sep).join('/'));
          summary.jsonl[capped.text.includes('\r\n') ? 'crlf' : 'lf']++;
          const r = scrubJsonl(capped.text, scrubPath);
          summary.unparsableLines += r.bad;
          result = r.text;
        } else if (src.endsWith('.json')) result = scrubJson(text, scrubPath);
        else if (src.endsWith('.md')) result = scrubMarkdown(text, scrubPath);
        else if (rel.split(path.sep)[0] === 'debug') result = scrubDebugLog(text, scrubPath);
        else result = filler(scrubPath(text)); // file-history backups and anything else: shape only
      } catch {
        result = filler(scrubPath(text));
      }
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.writeFileSync(dest, result);
      summary.files++;
    }
  };
  walk(home);
  const projectsOut = path.join(destRoot, 'projects');
  if (fs.existsSync(projectsOut)) {
    for (const name of fs.readdirSync(projectsOut)) {
      const tail = name.split(/Users-fixture-?|home-fixture-?/)[1] ?? '';
      const unknown = tail.split('-').filter((t) => t && !GENERIC_SEGMENTS.has(t) && !/^(proj|\d+)$/.test(t));
      if (unknown.length) summary.unmappedFolders.push(name);
    }
  }
  if (config && fs.existsSync(config)) {
    fs.mkdirSync(out, { recursive: true });
    fs.writeFileSync(path.join(out, 'claude.json'), scrubJson(fs.readFileSync(config, 'utf8'), scrubPath));
    summary.files++;
  }
  return summary;
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) args[argv[i].replace(/^--/, '')] = argv[i + 1];
  return args;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const a = parseArgs(process.argv.slice(2));
  if (!a.home || !a.out) {
    console.error('usage: node scripts/scrub-claude-home.mjs --home <~/.claude> [--config <~/.claude.json>] --out <dir> [--user <name>] [--max-bytes N]');
    process.exit(2);
  }
  const summary = scrubTree({
    home: path.resolve(a.home), config: a.config && path.resolve(a.config), out: path.resolve(a.out),
    user: a.user || os.userInfo().username, maxBytes: a['max-bytes'] ? Number(a['max-bytes']) : undefined,
  });
  console.log(JSON.stringify(summary, null, 2));
}
