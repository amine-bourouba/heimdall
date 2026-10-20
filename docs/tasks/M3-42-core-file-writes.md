# M3-42 · Core writes for agent, command, skill and output-style files

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

Agents save (renaming deletes the old file), templates are scaffolded with unique names, config items delete by ConfigRef, and agents import and export as JSON, all confined to known roots.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift:650-862`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`

## Outputs

- `crates/heimdall-core/src/config/write/files.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Paths are canonicalised and must stay inside the resolved root; symlink escapes are refused (test).
- [ ] Deleting a skill removes its folder; template names follow `new-agent`, `new-agent-2`, …

## Dependencies

- M1-22 — Safe JSON write primitive for Claude Code's files
- M1-24 — Loaders for commands, skills, plans, output styles and agents

## Owning agent

rust-engineer

## Reviewer agent

security-auditor, architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
