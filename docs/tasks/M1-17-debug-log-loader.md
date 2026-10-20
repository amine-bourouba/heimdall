# M1-17 · Debug log loader

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

The debug-log module pages `~/.claude/debug/<sessionId>.txt` entries of the form `<ISO8601> [LEVEL] message`, as DebugLogLoader does.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/DebugLogLoader.swift`
- `$SWIFT_REF/PoirotTests/Services/DebugLogLoaderTests.swift`

## Outputs

- `crates/heimdall-core/src/data/debug_log.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over DebugLogLoaderTests pass (paging with offset and limit, multi-line entries, allSessionIds).

## Dependencies

- M1-07 — Streaming JSONL reader with CRLF and truncation tolerance
- M1-09 — Serde models for stats, history, todos, facets, debug log and file history

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
