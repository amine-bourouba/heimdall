# M3-52 · Platform execution of launch actions

> Milestone M3 · Workstream: Tauri shell / OS integration · Scope: Later

## Goal

A platform module runs validated launch actions: argument-array spawns without a shell, editor URL schemes and reveal-in-file-manager, with the right per-OS behaviour.

## Inputs

- `docs/adr/0003-platform-abstraction.md` — Launchers
- `docs/adr/0010-webview-trust-boundary.md` — Process launch
- `docs/spikes/editor-url-schemes.md`

## Outputs

- `src-tauri/src/platform/mod.rs` — modify
- `src-tauri/src/platform/windows.rs` — modify
- `src-tauri/src/platform/macos.rs` — modify
- `src-tauri/src/platform/linux.rs` — modify

## Acceptance criteria

- [ ] Spawns use `std::process::Command` with argument arrays and detach; Windows uses CREATE_NO_WINDOW; `.bat` and `.cmd` targets are refused.
- [ ] URL schemes and reveal go through the opener plugin's Rust API; Linux reveal degrades to opening the parent folder.

## Dependencies

- M1-03 — Scaffold the Tauri app crate with final identity, CSP and module stubs
- M1-32 — Validated argv builders for terminals and editors
- M3-54 — Verify editor URL schemes per OS

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, security-auditor, deployment-engineer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
