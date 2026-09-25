# M1-07 · Streaming JSONL reader with CRLF and truncation tolerance

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A streaming reader yields JSON records line by line, trims a trailing `\r`, skips unparsable lines, tolerates a truncated final line, and reports how many lines it skipped.

## Inputs

- `$POIROT/Poirot/Sources/Services/TranscriptParser.swift:10-30, 140-215`
- `docs/adr/0005-error-model.md`

## Outputs

- `crates/heimdall-core/src/jsonl.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Tests cover LF, CRLF, mixed endings, blank lines, an invalid line mid-file, and a truncated last line; skipped counts are asserted.
- [ ] Reading uses `BufReader` line iteration; no `read_to_string` on transcript files (reviewer checks).
- [ ] `read_head(path, max_lines, chunk = 8192)` reads in 8 KiB chunks and stops after N qualifying lines (used by the header tier).

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
