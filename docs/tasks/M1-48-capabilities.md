# M1-48 · Per-window capability files

> Milestone M1 · Workstream: Security · Scope: MVP

## Goal

The main and tray windows each get one capability file granting only the permissions ADR 0010 lists.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Capabilities
- `docs/ipc-contract.md` — §8 Window access
- `docs/security/command-exposure.md`

## Outputs

- `src-tauri/capabilities/main.json` — create
- `src-tauri/capabilities/tray.json` — create

## Acceptance criteria

- [ ] main.json grants the app commands marked `main` in the contract, the event permissions for listening, `clipboard-manager:allow-write-text`, `clipboard-manager:allow-write-image`, and the window permission needed for the macOS drag region; nothing else.
- [ ] tray.json grants only `get_tray_snapshot`, `open_in_main_window` and `get_prefs`.
- [ ] Neither file contains `fs:`, `shell:`, `opener:allow-open-path` or a remote URL.
- [ ] A manual check from the tray webview devtools shows `update_prefs` is refused; the result goes in the progress log.

## Dependencies

- M1-49 — Scope app commands with AppManifest and verify exposure

## Owning agent

security-engineer

## Reviewer agent

security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
