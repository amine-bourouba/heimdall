# M1-35 · Line diff port

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A Rust port of LineDiff produces the same diff lines, with the 5000-line cap flag used by the diff view.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Utilities/LineDiff.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Session/EditDiffView.swift:22`
- `$SWIFT_REF/PoirotTests/Utilities/LineDiffTests.swift`

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

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
