# M1-13 · Transcript parser: header and summary tiers

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

The header tier (first 50 qualifying lines read in 8 KiB chunks) and the summary tier (single pass, no message materialisation) return the same fields as parseHeader and parseSummary.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/TranscriptParser.swift:140-354`
- `$SWIFT_REF/PoirotTests/Services/TranscriptParserTests.swift`
- `docs/spec/session-views.md`

## Outputs

- `crates/heimdall-core/src/transcript/header.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/transcript/summary.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over header and summary tests pass.
- [ ] A test asserts that summary token totals equal totals computed from the full parse for every fixture transcript.
- [ ] The summary tier never builds a `Vec<Message>` (reviewer checks; M1-57 measures).

## Dependencies

- M0-08 — Behaviour spec for the session browser and conversation view
- M1-12 — Transcript parser: full-parse tier, blocks, system content, tool input

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
