// Linux real-app smoke (port-plan.md P-8, this task's AC3): tauri-driver is the WebDriver server
// Tauri itself documents for this (it proxies WebDriver commands to WebKitGTK's own native
// WebDriver support, which Playwright cannot speak -- Playwright only drives browsers it controls
// directly or over CDP, and WebKitGTK exposes neither to it). This is Tauri's own documented
// WebdriverIO config shape, adapted to this repo's real binary path and `HEIMDALL_CLAUDE_HOME`
// fixture override (M1-38) instead of a made-up one.
//
// Cannot be executed on this dev machine: tauri-driver only exists for Linux/WebKitGTK (A-10 --
// no macOS WebDriver at all; Windows gets `tauriCdp.ts` instead, which needs no WebDriver server).
// Real verification is Linux CI, under `xvfb-run` (no real display in a CI container).
import { spawn, spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import os from 'node:os';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BINARY_PATH = path.resolve(__dirname, '../../src-tauri/target/debug/heimdall-desktop');
// `HEIMDALL_E2E_CLAUDE_HOME` (this test harness's own env var, unrelated to the app's own
// `HEIMDALL_CLAUDE_HOME` read below) lets `npm run test:e2e:linux-smoke:security` (M2-26) point
// this same spec at `fixtures/security/malicious-claude-home` instead, without a second wdio
// config or spec file duplicating this one almost entirely.
const CLAUDE_HOME = process.env.HEIMDALL_E2E_CLAUDE_HOME
  ? path.resolve(__dirname, '../..', process.env.HEIMDALL_E2E_CLAUDE_HOME)
  : path.resolve(__dirname, '../../fixtures/screenshot/claude-home');

let tauriDriver;

export const config = {
  specs: ['./app.smoke.mjs'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: BINARY_PATH,
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: { ui: 'bdd', timeout: 60000 },

  // The binary has to already exist -- this task does not own building it (`cargo build`, a
  // separate CI/release step); failing fast with a clear message beats tauri-driver's own
  // opaque "could not start session" error for a missing executable.
  onPrepare: () => {
    const result = spawnSync('test', ['-x', BINARY_PATH]);
    if (result.status !== 0) {
      throw new Error(`Expected a debug build at ${BINARY_PATH} -- run \`cargo build\` in src-tauri/ first.`);
    }
  },

  // tauri-driver proxies WebDriver requests to the app; the app's own `HEIMDALL_CLAUDE_HOME`
  // override (debug builds only, state.rs) reaches it through this spawned process's own
  // environment, which child processes inherit by default.
  beforeSession: () => {
    tauriDriver = spawn(path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'), [], {
      stdio: [null, process.stdout, process.stderr],
      env: { ...process.env, HEIMDALL_CLAUDE_HOME: CLAUDE_HOME },
    });
  },

  afterSession: () => tauriDriver?.kill(),
};
