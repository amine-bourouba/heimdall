# M3-40 · Core writes for settings.json

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

Plugin toggle/remove, MCP permission stripping, default and project model, and hook save/delete/import go through the safe-write primitive and never fall back to an empty document or to the global file.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/SettingsWriter.swift:14-82, 198-340`
- `$POIROT/PoirotTests/Services/SettingsWriterMCPTests.swift`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`

## Outputs

- `crates/heimdall-core/src/config/write/settings.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over SettingsWriterMCPTests pass.
- [ ] For every operation, a corrupt target yields `parse` and no bytes change.
- [ ] A project-scope write with no resolvable project returns `invalidInput` (Poirot silently wrote the global file, SettingsWriter.swift:318-326).
- [ ] Hook handlers keep unknown fields across save.
- [ ] Setting the default or project model writes a model id or alias (for example `claude-opus-4-8` or `opus`), never a display name such as `Opus 4.8` (port-plan C-27).

## Dependencies

- M1-22 — Safe JSON write primitive for Claude Code's files
- M1-25 — Settings and hooks readers

## Owning agent

rust-engineer

## Reviewer agent

security-auditor, architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
