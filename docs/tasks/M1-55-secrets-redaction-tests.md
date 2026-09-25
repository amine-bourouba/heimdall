# M1-55 · Secrets never reach DTOs or debug output

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

A test proves that no secret value from MCP env/headers reaches any serialised DTO or `Debug` output.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Secrets
- `fixtures/synthetic/`

## Outputs

- `crates/heimdall-core/tests/secrets_redaction.rs` — create
- `fixtures/security/secrets-claude.json` — create

## Acceptance criteria

- [ ] Sentinel values `SENTINEL-SECRET-*` never appear in DTO JSON or `format!("{:?}")` output for every config loader.

## Dependencies

- M0-11 — Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree
- M1-10 — Config models, catalogs and the redacting Secret type
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers

## Owning agent

test-automator

## Reviewer agent

qa-expert, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
