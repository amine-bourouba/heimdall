# M3-45 · MCP write commands with native confirmations

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

MCP add, update and remove show a native confirmation listing the exact command, each argument, env keys, URL and header keys before writing.

## Inputs

- `docs/ipc-contract.md` — commands/mcp.rs
- `docs/adr/0010-webview-trust-boundary.md` — Code-execution writes, Secrets

## Outputs

- `src-tauri/src/commands/mcp.rs` — modify: writes

## Acceptance criteria

- [ ] Secret values are never shown in the dialog or returned.
- [ ] Declining returns `cancelled` and writes nothing.

## Dependencies

- M1-45 — MCP read commands
- M3-41 — Core writes for MCP servers in ~/.claude.json

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
