# M1-35 · Line diff port

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A Rust port of LineDiff produces the same diff lines, with the 5000-line cap flag used by the diff view.

## Inputs

- `$POIROT/Poirot/Sources/Utilities/LineDiff.swift`
- `$POIROT/Poirot/Sources/Views/Session/EditDiffView.swift:22`
- `$POIROT/PoirotTests/Utilities/LineDiffTests.swift`

## Outputs

- `crates/heimdall-core/src/diff.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over LineDiffTests pass; inputs over 5000 lines return `truncated: true`.

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
