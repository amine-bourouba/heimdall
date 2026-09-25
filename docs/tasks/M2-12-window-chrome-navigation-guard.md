# M2-12 · Per-OS window chrome and navigation guard

> Milestone M2 · Workstream: Tauri shell / OS integration · Scope: MVP

## Goal

The main window is created in Rust with per-OS chrome (macOS overlay title bar with traffic lights, native decorations on Windows and Linux), and the webview cannot navigate away from the app origin or open new windows.

## Inputs

- `$POIROT/Poirot/Sources/App/PoirotApp.swift:38-46`
- `docs/port-plan.md` — decision P-4
- `docs/adr/0010-webview-trust-boundary.md` — Links

## Outputs

- `src-tauri/src/window.rs` — modify
- `src-tauri/tauri.conf.json` — modify: move window definition into window.rs

## Acceptance criteria

- [ ] macOS: overlay title bar, hidden title, drag region; Windows and Linux: native decorations.
- [ ] Navigation to any origin other than the app's is refused; new-window requests are refused (verify the handler API in the pinned Tauri version and record it in the progress log).
- [ ] Manual check on each OS recorded in the progress log; M2-26 automates the check.

## Dependencies

- M1-03 — Scaffold the Tauri app crate with final identity, CSP and module stubs

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, security-auditor, deployment-engineer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
