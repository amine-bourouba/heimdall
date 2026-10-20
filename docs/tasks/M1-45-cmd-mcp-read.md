# M1-45 · MCP read commands

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

`list_mcp_servers`, `get_mcp_statuses` and `get_mcp_catalog` return redacted server data and statuses.

## Inputs

- `docs/ipc-contract.md` — commands/mcp.rs
- `docs/adr/0010-webview-trust-boundary.md` — Secrets

## Outputs

- `src-tauri/src/commands/mcp.rs` — modify

## Acceptance criteria

- [ ] No response carries an env or header value (test with sentinel secrets).

## Dependencies

- M1-26 — MCP server (five sources) and plugin readers
- M1-28 — MCP process status on sysinfo
- M1-38 — App state and the (mtime, size)-validated cache

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
