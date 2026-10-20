# M1-32 · Validated argv builders for terminals and editors

> Milestone M1 · Workstream: Rust data core · Scope: Later

## Goal

Pure builders produce argument arrays or editor URLs for every supported terminal and editor on each OS, and reject unsafe inputs.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Utilities/TerminalLauncher.swift`
- `$SWIFT_REF/Poirot/Sources/Utilities/EditorLauncher.swift`
- `$SWIFT_REF/Poirot/Sources/Models/PreferredTerminal.swift`
- `$SWIFT_REF/Poirot/Sources/Models/PreferredEditor.swift`
- `docs/adr/0003-platform-abstraction.md` — Launchers
- `docs/adr/0010-webview-trust-boundary.md` — Process launch

## Outputs

- `crates/heimdall-core/src/os_integration/launch.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] No builder returns a shell string; outputs are `Argv(Vec<String>)` or `Url(String)`.
- [ ] `validate_launch_path` rejects relative paths, a leading `-`, UNC paths and non-regular files (facts injected by the caller).
- [ ] Session ids must match the UUID pattern or the `agent-` form from docs/ipc-contract.md.
- [ ] Cross-OS table tests cover spaces, `;&|%^"`, a leading `-`, UNC paths and non-UUID ids; Windows Terminal output never lets `;` split commands.
- [ ] Editors prefer URL schemes (`vscode://file/<abs>:<line>`, `cursor://…`, `zed://…`) where M3-54 confirms them; `.cmd` shims are never spawned.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-11 — Typed Prefs model with per-OS defaults
- M1-31 — Cross-platform PATH search for claude and editor CLIs

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
