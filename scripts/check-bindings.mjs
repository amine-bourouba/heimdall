#!/usr/bin/env node
// Regenerate src/ipc/bindings.ts from the tauri-specta builder and fail if it differs from what is
// checked in (task M1-37, ADR 0004: "CI regenerates and diffs").
//
//   node scripts/check-bindings.mjs
//
// Runs `cargo run -p heimdall-desktop --bin generate_bindings`, which calls
// `commands::builder().export(...)` directly (no window, no event loop) and overwrites
// src/ipc/bindings.ts in place; this script then diffs that file against its prior content.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const bindingsPath = path.join(repoRoot, 'src', 'ipc', 'bindings.ts');

const before = fs.existsSync(bindingsPath) ? fs.readFileSync(bindingsPath, 'utf8') : null;

execFileSync('cargo', ['run', '--quiet', '-p', 'heimdall-desktop', '--bin', 'generate_bindings'], {
  cwd: repoRoot,
  stdio: 'inherit',
});

const after = fs.readFileSync(bindingsPath, 'utf8');

if (before === after) {
  console.log('src/ipc/bindings.ts is up to date.');
  process.exit(0);
}

console.error('src/ipc/bindings.ts is out of date: regenerating it produced a different file.');
console.error('Run `node scripts/check-bindings.mjs` locally and commit the result.');
process.exit(1);
