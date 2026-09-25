# M3-59 · Grant launch-at-login permissions

> Milestone M3 · Workstream: Security · Scope: Later

## Goal

The main window may read and toggle launch-at-login through the autostart plugin, and the security lint allows exactly those permissions.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Capabilities

## Outputs

- `src-tauri/capabilities/main.json` — modify
- `scripts/lint-security-config.mjs` — modify

## Acceptance criteria

- [ ] Only `autostart:allow-enable`, `autostart:allow-disable` and `autostart:allow-is-enabled` are added.

## Dependencies

- M1-48 — Per-window capability files
- M2-27 — Security configuration lint

## Owning agent

security-engineer

## Reviewer agent

security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
