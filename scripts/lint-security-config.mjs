// A script fails CI when the CSP, capabilities, plugin set or Tauri security flags drift from
// ADR 0010 (M2-27). Every check below is one `docs/adr/0010-webview-trust-boundary.md` point 3/4/
// 10 sentence, turned into code instead of staying a human-reviewed string: `tauri.conf.json`,
// `capabilities/*.json` and `Cargo.toml` can all be hand-edited by a future task, and nothing
// else re-checks these exact facts on every commit.
//
// Each `lint*` function is pure (no filesystem access) so `lint-security-config.test.mjs` can
// feed it a deliberately-broken fixture per rule, independent of this repo's own current,
// currently-compliant files.
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

// The exact strings ADR 0010 point 3 specifies -- not "equivalent" or "reordered", since a CSP's
// own directive order has no semantic meaning to a reviewer eyeballing a diff against this exact
// string, which is the whole point of pinning it byte-for-byte.
export const EXPECTED_CSP =
  "default-src 'self'; script-src 'self'; img-src 'self' data: blob:; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'";
export const EXPECTED_DEV_CSP =
  "default-src 'self'; script-src 'self' http://localhost:*; style-src 'self' 'unsafe-inline' http://localhost:*; img-src 'self' data: blob: http://localhost:*; connect-src ipc: http://ipc.localhost http://localhost:* ws://localhost:*; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'";

// ADR 0010 point 4: "Tray: only get_tray_snapshot, open_in_main_window, get_prefs."
export const TRAY_PERMISSION_ALLOWLIST = ['allow-get-tray-snapshot', 'allow-open-in-main-window', 'allow-get-prefs'];

// M3-59's own AC: "Only autostart:allow-enable, autostart:allow-disable and
// autostart:allow-is-enabled are added" -- unlike the tray allowlist above, this applies to every
// capability file, not one specific window, since there is no legitimate reason any window would
// ever need a broader autostart grant (e.g. a hypothetical future `autostart:default`).
export const AUTOSTART_PERMISSION_ALLOWLIST = ['autostart:allow-enable', 'autostart:allow-disable', 'autostart:allow-is-enabled'];

/** @param {unknown} conf parsed `tauri.conf.json` @returns {string[]} violations, empty if clean */
export function lintTauriConf(conf) {
  const violations = [];
  const app = conf?.app ?? {};
  const security = app.security ?? {};

  if (security.csp !== EXPECTED_CSP) violations.push('app.security.csp does not match ADR 0010\'s exact production CSP string');
  if (security.devCsp !== EXPECTED_DEV_CSP) violations.push('app.security.devCsp does not match ADR 0010\'s exact dev CSP string');
  if (typeof security.csp === 'string' && security.csp.includes('unsafe-eval')) violations.push('app.security.csp contains unsafe-eval');
  if (typeof security.devCsp === 'string' && security.devCsp.includes('unsafe-eval')) violations.push('app.security.devCsp contains unsafe-eval');
  if (typeof security.csp === 'string' && security.csp.includes('unsafe-inline')) violations.push('app.security.csp (production) contains unsafe-inline');
  if (security.freezePrototype !== true) violations.push('app.security.freezePrototype is not true');
  if (security.assetProtocol?.enable !== false) violations.push('app.security.assetProtocol.enable is not false');
  if (app.withGlobalTauri !== false) violations.push('app.withGlobalTauri is not false');

  return violations;
}

/** A capability's `permissions` entries are each either a plain identifier string or an object
 * with an `identifier` field (Tauri's own scoped-permission form, e.g. a path-restricted `fs:`
 * grant) -- normalise to the identifier string either way. */
function permissionIdentifier(permission) {
  return typeof permission === 'string' ? permission : permission?.identifier;
}

/** @param {unknown} capability one parsed `capabilities/*.json` file @returns {string[]} */
export function lintCapability(capability) {
  const violations = [];
  const permissions = Array.isArray(capability?.permissions) ? capability.permissions : [];
  const identifiers = permissions.map(permissionIdentifier).filter((id) => typeof id === 'string');

  for (const identifier of identifiers) {
    if (identifier.startsWith('fs:')) violations.push(`permission "${identifier}" grants the fs plugin`);
    if (identifier.startsWith('shell:')) violations.push(`permission "${identifier}" grants the shell plugin`);
    if (identifier.startsWith('opener:allow-open-path')) violations.push(`permission "${identifier}" grants opener:allow-open-path`);
    if (identifier.startsWith('autostart:') && !AUTOSTART_PERMISSION_ALLOWLIST.includes(identifier)) {
      violations.push(`permission "${identifier}" grants an autostart permission outside its allowlist (${AUTOSTART_PERMISSION_ALLOWLIST.join(', ')})`);
    }
  }

  if (capability?.remote !== undefined) violations.push('capability declares a "remote" key, allowing remote URLs to use its permissions');

  if (Array.isArray(capability?.windows) && capability.windows.includes('tray')) {
    for (const identifier of identifiers) {
      if (!TRAY_PERMISSION_ALLOWLIST.includes(identifier)) {
        violations.push(`tray capability grants "${identifier}", outside its allowlist (${TRAY_PERMISSION_ALLOWLIST.join(', ')})`);
      }
    }
  }

  return violations;
}

/** @param {string} cargoTomlText raw `Cargo.toml` contents @returns {string[]} */
export function lintCargoToml(cargoTomlText) {
  const violations = [];
  for (const line of cargoTomlText.split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith('#')) continue;
    if (/^tauri-plugin-fs\s*=/.test(trimmed)) violations.push('Cargo.toml depends on tauri-plugin-fs');
    if (/^tauri-plugin-shell\s*=/.test(trimmed)) violations.push('Cargo.toml depends on tauri-plugin-shell');
  }
  return violations;
}

/** Reads this repo's real files and runs every rule above against them. @returns {string[]} */
export function lintRepo(root = ROOT) {
  const violations = [];

  const conf = JSON.parse(readFileSync(path.join(root, 'src-tauri/tauri.conf.json'), 'utf-8'));
  for (const v of lintTauriConf(conf)) violations.push(`tauri.conf.json: ${v}`);

  const capabilitiesDir = path.join(root, 'src-tauri/capabilities');
  for (const file of readdirSync(capabilitiesDir).filter((f) => f.endsWith('.json'))) {
    const capability = JSON.parse(readFileSync(path.join(capabilitiesDir, file), 'utf-8'));
    for (const v of lintCapability(capability)) violations.push(`capabilities/${file}: ${v}`);
  }

  const cargoToml = readFileSync(path.join(root, 'src-tauri/Cargo.toml'), 'utf-8');
  for (const v of lintCargoToml(cargoToml)) violations.push(`Cargo.toml: ${v}`);

  return violations;
}

function main() {
  const violations = lintRepo();
  if (violations.length > 0) {
    console.error('Security configuration drift from ADR 0010:');
    for (const v of violations) console.error(`  - ${v}`);
    process.exitCode = 1;
  } else {
    console.log('OK: tauri.conf.json, capabilities/ and Cargo.toml match ADR 0010.');
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
