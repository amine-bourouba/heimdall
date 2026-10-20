# M2-30 · M2 exit gate review

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

The M2 exit criteria are checked on Windows and Linux against the Swift reference on the shared fixture set, and the session rows of the parity matrix are updated.

## Inputs

- `docs/port-plan.md` — §7 M2 gate
- `docs/parity-matrix.md`
- Reference screenshots from the golden-oracle workflow's optional job

## Outputs

- `docs/parity-matrix.md` — modify: sections B and C session rows

## Acceptance criteria

- [ ] Side-by-side review of each session screen against the Swift reference screenshots is recorded with a verdict per OS.
- [ ] Parity rows show per-OS status; every Degraded row names its reason.

## Dependencies

- M1-59 — M1 exit gate review
- M2-11 — open_external_url command
- M2-24 — Visual baselines for the session screens
- M2-26 — Malicious transcript end-to-end test
- M2-29 — Frontend, security and E2E jobs in CI

## Owning agent

test-automator

## Reviewer agent

qa-expert, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
