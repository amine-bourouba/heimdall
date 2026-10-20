# M3-44 · Config write commands with native confirmations

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

Config write commands mutate settings and config files through core, showing a native confirmation with exact values before any hook command or URL is written or imported.

## Inputs

- `docs/ipc-contract.md` — commands/config_write.rs
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `docs/adr/0010-webview-trust-boundary.md` — Code-execution writes

## Outputs

- `src-tauri/src/commands/config_write.rs` — modify

## Acceptance criteria

- [ ] Hook save and import show the exact command or URL in a Rust-side dialog; declining returns `cancelled` and writes nothing.
- [ ] Import and export use Rust-opened file dialogs.
- [ ] Every command returns `WriteOutcome`; tests use an injected dialog.

## Dependencies

- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M1-38 — App state and the (mtime, size)-validated cache
- M3-40 — Core writes for settings.json
- M3-42 — Core writes for agent, command, skill and output-style files

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
