# M1-15 · Prompt history reader

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

The history module reads `~/.claude/history.jsonl` in pages with stable per-entry ids, matching HistoryLoader.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/HistoryLoader.swift`
- `$SWIFT_REF/PoirotTests/Services/HistoryLoaderTests.swift`
- `docs/ipc-contract.md` — HistoryEntryId

## Outputs

- `crates/heimdall-core/src/data/history.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over HistoryLoaderTests pass (ms-epoch timestamps, pastedContents, entryCount).
- [ ] Entry ids come from a hash of the raw line plus its timestamp, so a later delete can find the line again.

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
