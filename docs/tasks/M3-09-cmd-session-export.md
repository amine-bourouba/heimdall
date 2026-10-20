# M3-09 · Session export commands

> Milestone M3 · Workstream: Rust data core · Scope: MVP

## Goal

`export_session` writes Markdown to the clipboard or to a file chosen in a Rust-opened save dialog, and `export_edit_patch` saves a unified diff for one Edit block.

## Inputs

- `docs/ipc-contract.md` — commands/export.rs
- `docs/adr/0010-webview-trust-boundary.md`
- `$SWIFT_REF/Poirot/Sources/Services/Export/SessionExporter.swift:280-346`

## Outputs

- `src-tauri/src/commands/export.rs` — modify: session exports

## Acceptance criteria

- [ ] Cancelling the dialog returns `cancelled`; the webview never supplies a path.
- [ ] Tests use an injected dialog and clipboard.

## Dependencies

- M1-36 — Session Markdown and patch export
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M1-41 — Session read commands

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
