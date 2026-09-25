// Tests for scripts/scrub-claude-home.mjs (task M0-13). Run: node --test scripts/scrub-claude-home.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { scrubTree, filler, makePathScrubber, encodeSegment } from './scrub-claude-home.mjs';

const USER = 'alice';

function write(p, content) {
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, content);
}

function allFiles(dir) {
  return fs.readdirSync(dir, { withFileTypes: true, recursive: true })
    .filter((e) => e.isFile())
    .map((e) => path.join(e.parentPath ?? e.path, e.name));
}

function buildSource() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'scrub-src-'));
  const home = path.join(root, '.claude');
  const projects = path.join(home, 'projects');
  const userLine = JSON.stringify({
    type: 'user', uuid: 'u1', sessionId: 's1', timestamp: '2026-09-01T10:00:00.000Z', gitBranch: 'feature/acme-login',
    cwd: 'C:\\Users\\alice\\src\\app', message: { role: 'user', content: 'Please fix my secret project' },
    filePath: 'C:\\Users\\alice\\.claude\\projects\\C--Users-alice-src-app\\memory\\m.md',
  });
  const assistantLine = JSON.stringify({
    type: 'assistant', uuid: 'a1', sessionId: 's1', timestamp: '2026-09-01T10:00:05.000Z',
    message: {
      role: 'assistant', model: 'claude-fable-5-1', usage: { input_tokens: 12, output_tokens: 34 },
      content: [
        { type: 'text', text: 'Sure, alice. Line one\nLine two' },
        { type: 'tool_use', id: 't1', name: 'Edit', input: { file_path: 'C:\\Users\\alice\\src\\app\\main.rs', old_string: 'a\nb', new_string: 'c\nd\ne' } },
      ],
    },
  });
  write(path.join(projects, 'C--Users-alice-src-app', 's1.jsonl'), `${userLine}\r\n${assistantLine}\r\n{"type":"user","trunc`);
  write(path.join(projects, '-Users-alice-src-web', 's2.jsonl'),
    `${JSON.stringify({ type: 'user', cwd: '/Users/alice/src/web', message: { content: 'hi alice@corp.com' } })}\n`);
  write(path.join(projects, '-home-alice-work', 's3.jsonl'),
    `${JSON.stringify({ type: 'user', cwd: '/home/alice/work', message: { content: 'x' } })}\n`);
  write(path.join(projects, 'C--Users-alice-Desktop-Acme-Corp---Panel', 's4.jsonl'),
    `${JSON.stringify({ type: 'user', cwd: 'C:\\Users\\alice\\Desktop\\Acme Corp - Panel', message: { content: 'y' } })}\n`);
  write(path.join(projects, 'C--Users-alice-Desktop-Globex', 's5.jsonl'),
    `${JSON.stringify({ type: 'user', message: { content: 'no cwd here' } })}\n`);
  write(path.join(home, 'settings.json'), JSON.stringify({ model: 'opus', env: { API_KEY: 'sk-live-123' }, hooks: {} }, null, 2) + '\n');
  write(path.join(home, 'history.jsonl'), `${JSON.stringify({ display: 'deploy with token abc', timestamp: 1, project: '/Users/alice/src/web' })}\n`);
  write(path.join(home, 'debug', 's1.txt'), '2026-09-01T10:00:00.000Z [DEBUG] loaded /Users/alice/x\n');
  write(path.join(home, 'agents', 'helper.md'), '---\nname: helper\nmodel: sonnet\ndescription: private stuff\n---\nBody text for alice\n');
  write(path.join(home, 'skills', 'synced', 'deadbeef-1111-4222-8333-444444444444_feedface-5555-4666-8777-888888888888', 'tidy', 'SKILL.md'),
    '---\nname: tidy\ndescription: tidy up\n---\nBody\n');
  write(path.join(home, '.credentials.json'), JSON.stringify({ claudeAiOauth: { accessToken: 'sk-ant-oat01-secret' } }));
  write(path.join(home, 'shell-snapshots', 'snap.sh'), 'export SECRET=1');
  const config = path.join(root, '.claude.json');
  write(config, JSON.stringify({
    oauthAccount: { emailAddress: 'alice@corp.com' },
    mcpServers: { github: { command: 'npx', args: ['-y', 'server'], env: { GITHUB_TOKEN: 'ghp_realtoken' } } },
    projects: { 'C:\\Users\\alice\\src\\app': { mcpServers: {} } },
    githubRepoPaths: { 'alice-gh/secret-repo': ['C:\\Users\\alice\\src\\app'] },
    passesEligibilityCache: { 'DEADBEEF-1111-4222-8333-444444444444': { eligible: false } },
    overageCreditGrantCache: { 'deadbeef-1111-4222-8333-444444444444': { info: {} } },
  }, null, 2));
  return { root, home, config };
}

// Pseudonyms are assigned in sorted order of the non-generic segments: 'Acme Corp - Panel', 'app', 'web'.
const WIN = 'C--Users-fixture-src-proj-2';
const CLIENT = 'C--Users-fixture-Desktop-proj-1';

function run() {
  const src = buildSource();
  const out = fs.mkdtempSync(path.join(os.tmpdir(), 'scrub-out-'));
  const summary = scrubTree({ home: src.home, config: src.config, out, user: USER });
  return { src, out, summary };
}

const readLines = (out, folder, file, eol) =>
  fs.readFileSync(path.join(out, 'claude-home', 'projects', folder, file), 'utf8').split(eol);

test('CRLF line endings survive byte for byte', () => {
  const { out } = run();
  const text = fs.readFileSync(path.join(out, 'claude-home', 'projects', WIN, 's1.jsonl'), 'utf8');
  assert.equal((text.match(/\r\n/g) || []).length, 2);
  assert.equal((text.match(/(?<!\r)\n/g) || []).length, 0);
});

test('secrets under secret-looking keys are replaced', () => {
  const { out } = run();
  const settings = JSON.parse(fs.readFileSync(path.join(out, 'claude-home', 'settings.json'), 'utf8'));
  assert.equal(settings.env.API_KEY, '***scrubbed***');
  const cfg = JSON.parse(fs.readFileSync(path.join(out, 'claude.json'), 'utf8'));
  assert.equal(cfg.mcpServers.github.env.GITHUB_TOKEN, '***scrubbed***');
});

test('home and project names are replaced in Windows, macOS and Linux forms, including encoded folders', () => {
  const { out } = run();
  const projects = fs.readdirSync(path.join(out, 'claude-home', 'projects')).sort();
  assert.deepEqual(projects, ['-Users-fixture-src-proj-3', '-home-fixture-work', 'C--Users-fixture-Desktop-Globex', CLIENT, WIN]);
  const [user, assistant] = readLines(out, WIN, 's1.jsonl', '\r\n').map((l) => { try { return JSON.parse(l); } catch { return null; } });
  assert.equal(user.cwd, 'C:\\Users\\fixture\\src\\proj-2');
  // Encoded project folders inside a path get the same pseudonym as the plain segment.
  assert.equal(user.filePath, 'C:\\Users\\fixture\\.claude\\projects\\C--Users-fixture-src-proj-2\\memory\\m.md');
  assert.equal(assistant.message.content[1].input.file_path, 'C:\\Users\\fixture\\src\\proj-2\\main.rs');
  const cfg = JSON.parse(fs.readFileSync(path.join(out, 'claude.json'), 'utf8'));
  assert.deepEqual(Object.keys(cfg.projects), ['C:\\Users\\fixture\\src\\proj-2']);
  const client = JSON.parse(readLines(out, CLIENT, 's4.jsonl', '\n')[0]);
  assert.equal(client.cwd, 'C:\\Users\\fixture\\Desktop\\proj-1');
  assert.equal(encodeSegment('Acme Corp - Panel'), 'Acme-Corp---Panel');
  const linux = JSON.parse(readLines(out, '-home-fixture-work', 's3.jsonl', '\n')[0]);
  assert.equal(linux.cwd, '/home/fixture/work');
});

test('message text becomes same-length filler and JSON stays valid', () => {
  const { out } = run();
  const lines = readLines(out, WIN, 's1.jsonl', '\r\n');
  const user = JSON.parse(lines[0]);
  assert.equal(user.message.content.length, 'Please fix my secret project'.length);
  assert.notEqual(user.message.content, 'Please fix my secret project');
  assert.notEqual(user.gitBranch, 'feature/acme-login', 'branch names can identify clients');
  const assistant = JSON.parse(lines[1]);
  assert.equal(assistant.message.model, 'claude-fable-5-1');
  assert.equal(assistant.message.usage.output_tokens, 34);
  assert.equal(assistant.message.content[0].text.split('\n').length, 2, 'newlines inside text survive');
  assert.equal(assistant.message.content[1].input.new_string.split('\n').length, 3, 'diff shape survives');
  assert.throws(() => JSON.parse(lines[2]), 'a truncated last line stays unparsable');
});

test('credentials and non-allowlisted files are never copied', () => {
  const { out } = run();
  const names = allFiles(out).map((f) => path.relative(out, f).split(path.sep).join('/'));
  assert.ok(!names.some((n) => /credentials/i.test(n)));
  assert.ok(!names.some((n) => n.includes('shell-snapshots')));
});

test('the username, email and client names appear nowhere in the output', () => {
  const { out } = run();
  for (const f of allFiles(out)) {
    const rel = path.relative(out, f);
    const text = fs.readFileSync(f, 'utf8');
    assert.ok(!/alice/i.test(text) && !/alice/i.test(rel), `username leaked in ${rel}`);
    assert.ok(!/corp\.com/.test(text), `email leaked in ${rel}`);
    assert.ok(!/acme/i.test(text) && !/acme/i.test(rel), `client name leaked in ${rel}`);
  }
});

test('account UUID keys and GitHub repository keys are replaced', () => {
  const { out } = run();
  const text = fs.readFileSync(path.join(out, 'claude.json'), 'utf8');
  assert.ok(!/deadbeef/i.test(text) && !/secret-repo|alice-gh/.test(text));
  const cfg = JSON.parse(text);
  assert.deepEqual(Object.keys(cfg.githubRepoPaths), ['fixture/repo-1']);
  // The same id maps to the same fake in every cache, so links between caches survive.
  assert.deepEqual(Object.keys(cfg.passesEligibilityCache), Object.keys(cfg.overageCreditGrantCache));
  // Synced skill folders are named after the same ids and get the same fakes.
  const [synced] = fs.readdirSync(path.join(out, 'claude-home', 'skills', 'synced'));
  assert.ok(!/deadbeef|feedface/.test(synced));
  assert.ok(synced.split('_').includes(Object.keys(cfg.passesEligibilityCache)[0]));
});

test('folders whose project path is unknown are reported for review, not silently kept', () => {
  const { summary } = run();
  assert.deepEqual(summary.unmappedFolders, ['C--Users-fixture-Desktop-Globex']);
  assert.equal(summary.pseudonymisedSegments, 3);
});

test('markdown keeps frontmatter keys and model, fills the rest', () => {
  const { out } = run();
  const md = fs.readFileSync(path.join(out, 'claude-home', 'agents', 'helper.md'), 'utf8');
  assert.match(md, /^---\nname: \S+\nmodel: sonnet\ndescription: \S+ \S+\n---\n/);
});

test('debug log keeps timestamp and level prefix', () => {
  const { out } = run();
  const log = fs.readFileSync(path.join(out, 'claude-home', 'debug', 's1.txt'), 'utf8');
  assert.match(log, /^2026-09-01T10:00:00\.000Z \[DEBUG\] /);
});

test('helpers: filler keeps whitespace; path scrubber handles forward slashes', () => {
  assert.equal(filler('ab c\nd'), 'lo r\ne');
  assert.equal(makePathScrubber('bob')('C:/Users/bob/x'), 'C:/Users/fixture/x');
});
