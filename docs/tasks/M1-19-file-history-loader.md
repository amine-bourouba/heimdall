# M1-19 · File history loader with cross-OS file matching

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

The file-history module builds per-file version lists from transcript snapshot and delta records and matches files across Unix and Windows path forms.

## Inputs

- `$POIROT/Poirot/Sources/Services/FileHistoryLoader.swift`
- `$POIROT/PoirotTests/Services/FileHistoryLoaderTests.swift`
- `docs/adr/0003-platform-abstraction.md`

## Outputs

- `crates/heimdall-core/src/data/file_history.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over FileHistoryLoaderTests pass.
- [ ] Both `file-history-snapshot` and `file-history-delta` records are read (FileHistoryLoader.swift:121-141).
- [ ] File matching uses `paths::same_file`; a test with a Windows-path transcript and relative tracked names passes.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-07 — Streaming JSONL reader with CRLF and truncation tolerance
- M1-09 — Serde models for stats, history, todos, facets, debug log and file history

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
