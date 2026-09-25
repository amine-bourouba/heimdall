# M4-15 · Retire the Swift golden harness (conditional)

> Milestone M4 · Workstream: Legacy parity extraction · Scope: MVP

## Goal

Once the retirement condition holds, the Swift golden harness is removed while its outputs stay as regression fixtures.

## Inputs

- `docs/adr/0001-repo-layout-and-swift-reference.md` — Retirement condition
- `docs/parity-matrix.md`
- `docs/spec/test-carryover.md`

## Outputs

- `tools/golden-oracle/` — delete

## Acceptance criteria

- [ ] Precondition, checked and recorded: every MVP feature is At parity on Windows and Linux, and every parser-related PoirotTests row in test-carryover.md is covered by a passing Rust test.
- [ ] `fixtures/golden/` and `golden_parity.rs` remain and still pass.

## Dependencies

- M4-14 — MVP release sign-off

## Owning agent

swift-expert

## Reviewer agent

legacy-modernizer, qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
