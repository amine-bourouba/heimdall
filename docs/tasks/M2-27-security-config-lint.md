# M2-27 · Security configuration lint

> Milestone M2 · Workstream: Security · Scope: MVP

## Goal

A script fails CI when the CSP, capabilities, plugin set or Tauri security flags drift from ADR 0010.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/`
- `src-tauri/Cargo.toml`

## Outputs

- `scripts/lint-security-config.mjs` — create
- `scripts/lint-security-config.test.mjs` — create

## Acceptance criteria

- [ ] Fails on: CSP or devCsp different from the ADR strings; `unsafe-eval` in either, or `unsafe-inline` in the production CSP; `freezePrototype` not true; asset protocol enabled; `withGlobalTauri` true; any `fs:`, `shell:`, `opener:allow-open-path` or remote URL in a capability; a tray capability permission outside its allowlist; tauri-plugin-fs or tauri-plugin-shell in Cargo.toml.
- [ ] `node --test scripts/lint-security-config.test.mjs` passes with one failing fixture per rule.

## Dependencies

- M1-48 — Per-window capability files
- M2-12 — Per-OS window chrome and navigation guard

## Owning agent

security-engineer

## Reviewer agent

security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
