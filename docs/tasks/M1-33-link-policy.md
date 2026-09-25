# M1-33 · External URL allowlist

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

One function decides whether a URL may be opened externally: only http, https and mailto pass.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Links

## Outputs

- `crates/heimdall-core/src/os_integration/links.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] A corpus test rejects `file:`, `javascript:`, `data:`, custom schemes, UNC-style hosts, embedded credentials and over-long URLs, and accepts normal https and mailto links.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError

## Owning agent

rust-engineer

## Reviewer agent

security-auditor, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
