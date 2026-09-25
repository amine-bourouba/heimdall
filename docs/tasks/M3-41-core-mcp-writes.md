# M3-41 · Core writes for MCP servers in ~/.claude.json

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

Add, update and remove of top-level MCP servers preserve `type`, `url` and `headers`, take args as arrays, keep existing secrets unless replaced, and strip matching permissions on removal.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/SettingsWriter.swift:84-197`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `docs/adr/0010-webview-trust-boundary.md` — Secrets

## Outputs

- `crates/heimdall-core/src/config/write/mcp.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Update keeps fields the form does not edit (fix for SettingsWriter.swift:150-177).
- [ ] A `keep` secret marker leaves the stored value untouched; a new value replaces it.
- [ ] The line-number lookup for 'open at line' is ported with tests.

## Dependencies

- M1-22 — Safe JSON write primitive for Claude Code's files
- M1-26 — MCP server (five sources) and plugin readers
- M3-40 — Core writes for settings.json

## Owning agent

rust-engineer

## Reviewer agent

security-auditor, architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
