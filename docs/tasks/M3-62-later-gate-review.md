# M3-62 · Later-scope gate review

> Milestone M3 · Workstream: Testing · Scope: Later

## Goal

Every later-scope row of the parity matrix has a per-OS status with evidence and exactly one outcome where parity is not reached.

## Inputs

- `docs/parity-matrix.md`

## Outputs

- `docs/parity-matrix.md` — modify: later rows

## Acceptance criteria

- [ ] Evidence is recorded per row and OS in the progress log.

## Dependencies

- M3-35 — XSS tests for chart formatters
- M3-39 — Analytics PNG export and diff copy-as-image
- M3-50 — Write actions on plugins, models, commands, skills and output styles
- M3-51 — Delete actions for history, sessions and projects
- M3-55 — Editor, terminal and reveal buttons
- M3-56 — Tray icon and tray window
- M3-60 — Launch-at-login toggle
- M3-61 — Visual baselines for later-scope screens
- M4-14 — MVP release sign-off

## Owning agent

test-automator

## Reviewer agent

qa-expert, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
