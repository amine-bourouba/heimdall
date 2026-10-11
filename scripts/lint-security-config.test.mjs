// Tests for scripts/lint-security-config.mjs (task M2-27). Run: node --test scripts/lint-security-config.test.mjs
//
// Each rule gets one deliberately-broken fixture, cloned from a clean baseline that itself
// matches ADR 0010 exactly -- confirming each rule fires on the one thing it's meant to catch,
// and nothing else. `lintRepo()` is also run against this repo's own real files once, tying the
// whole script back to reality: a passing suite here and a clean repo could otherwise drift
// apart silently if every rule only ever saw synthetic fixtures.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { EXPECTED_CSP, EXPECTED_DEV_CSP, TRAY_PERMISSION_ALLOWLIST, lintTauriConf, lintCapability, lintCargoToml, lintRepo } from './lint-security-config.mjs';

function cleanConf() {
  return {
    app: {
      withGlobalTauri: false,
      security: {
        csp: EXPECTED_CSP,
        devCsp: EXPECTED_DEV_CSP,
        freezePrototype: true,
        assetProtocol: { enable: false },
      },
    },
  };
}

test('lintTauriConf: a clean, ADR-matching config produces no violations', () => {
  assert.deepEqual(lintTauriConf(cleanConf()), []);
});

test('lintTauriConf: fails when csp does not match the ADR string', () => {
  const conf = cleanConf();
  conf.app.security.csp = "default-src 'self'";
  assert.equal(lintTauriConf(conf).length, 1);
});

test('lintTauriConf: fails when devCsp does not match the ADR string', () => {
  const conf = cleanConf();
  conf.app.security.devCsp = "default-src 'self'";
  assert.equal(lintTauriConf(conf).length, 1);
});

test('lintTauriConf: fails when csp contains unsafe-eval', () => {
  const conf = cleanConf();
  conf.app.security.csp += " script-src 'unsafe-eval'";
  const violations = lintTauriConf(conf);
  assert.ok(violations.some((v) => v.includes('unsafe-eval')));
});

test('lintTauriConf: fails when devCsp contains unsafe-eval', () => {
  const conf = cleanConf();
  conf.app.security.devCsp += " script-src 'unsafe-eval'";
  const violations = lintTauriConf(conf);
  assert.ok(violations.some((v) => v.includes('devCsp') && v.includes('unsafe-eval')));
});

test('lintTauriConf: fails when the production csp contains unsafe-inline (devCsp is allowed to)', () => {
  const conf = cleanConf();
  conf.app.security.csp += " style-src 'unsafe-inline'";
  const violations = lintTauriConf(conf);
  assert.ok(violations.some((v) => v.includes('unsafe-inline')));
  // devCsp legitimately contains unsafe-inline (ADR 0010 point 3) and must never be flagged for it.
  assert.ok(!violations.some((v) => v.includes('devCsp') && v.includes('unsafe-inline')));
});

test('lintTauriConf: fails when freezePrototype is not true', () => {
  const conf = cleanConf();
  conf.app.security.freezePrototype = false;
  assert.equal(lintTauriConf(conf).length, 1);
});

test('lintTauriConf: fails when assetProtocol.enable is not false', () => {
  const conf = cleanConf();
  conf.app.security.assetProtocol.enable = true;
  assert.equal(lintTauriConf(conf).length, 1);
});

test('lintTauriConf: fails when withGlobalTauri is not false', () => {
  const conf = cleanConf();
  conf.app.withGlobalTauri = true;
  assert.equal(lintTauriConf(conf).length, 1);
});

test('lintCapability: a clean capability with only ordinary permissions produces no violations', () => {
  assert.deepEqual(lintCapability({ windows: ['main'], permissions: ['allow-get-prefs', 'core:event:allow-listen'] }), []);
});

test('lintCapability: fails on an fs: permission', () => {
  const violations = lintCapability({ windows: ['main'], permissions: ['fs:allow-read-text-file'] });
  assert.equal(violations.length, 1);
  assert.match(violations[0], /fs plugin/);
});

test('lintCapability: fails on a shell: permission', () => {
  const violations = lintCapability({ windows: ['main'], permissions: ['shell:allow-execute'] });
  assert.equal(violations.length, 1);
  assert.match(violations[0], /shell plugin/);
});

test('lintCapability: fails on opener:allow-open-path', () => {
  const violations = lintCapability({ windows: ['main'], permissions: ['opener:allow-open-path'] });
  assert.equal(violations.length, 1);
  assert.match(violations[0], /opener:allow-open-path/);
});

test('lintCapability: fails on a scoped-object permission form too, not just plain strings', () => {
  const violations = lintCapability({ windows: ['main'], permissions: [{ identifier: 'fs:allow-read-text-file', allow: [{ path: '$HOME/*' }] }] });
  assert.equal(violations.length, 1);
});

test('lintCapability: fails when a "remote" key is present', () => {
  const violations = lintCapability({ windows: ['main'], permissions: [], remote: { urls: ['https://*.example.com'] } });
  assert.equal(violations.length, 1);
  assert.match(violations[0], /remote/);
});

test('lintCapability: a tray capability within its own allowlist produces no violations', () => {
  assert.deepEqual(lintCapability({ windows: ['tray'], permissions: TRAY_PERMISSION_ALLOWLIST }), []);
});

test('lintCapability: fails when a tray capability grants a permission outside its allowlist', () => {
  const violations = lintCapability({ windows: ['tray'], permissions: ['allow-get-prefs', 'allow-list-projects'] });
  assert.equal(violations.length, 1);
  assert.match(violations[0], /allow-list-projects/);
});

test('lintCapability: the same out-of-allowlist permission on a non-tray window is not flagged by the tray rule', () => {
  const violations = lintCapability({ windows: ['main'], permissions: ['allow-list-projects'] });
  assert.deepEqual(violations, []);
});

test('lintCargoToml: a clean Cargo.toml with neither plugin produces no violations', () => {
  assert.deepEqual(lintCargoToml('[dependencies]\ntauri = "2.11.6"\ntauri-plugin-opener = "2.5.5"\n'), []);
});

test('lintCargoToml: fails when tauri-plugin-fs is a dependency', () => {
  const violations = lintCargoToml('[dependencies]\ntauri-plugin-fs = "2.0.0"\n');
  assert.equal(violations.length, 1);
  assert.match(violations[0], /tauri-plugin-fs/);
});

test('lintCargoToml: fails when tauri-plugin-shell is a dependency', () => {
  const violations = lintCargoToml('[dependencies]\ntauri-plugin-shell = "2.0.0"\n');
  assert.equal(violations.length, 1);
  assert.match(violations[0], /tauri-plugin-shell/);
});

test('lintCargoToml: a commented-out line is not a real dependency', () => {
  assert.deepEqual(lintCargoToml('# tauri-plugin-fs = "2.0.0"\n'), []);
});

test('lintRepo: this repo\'s own real tauri.conf.json, capabilities/ and Cargo.toml currently match ADR 0010', () => {
  assert.deepEqual(lintRepo(), []);
});
