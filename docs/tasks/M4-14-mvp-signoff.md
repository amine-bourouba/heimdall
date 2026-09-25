# M4-14 · MVP release sign-off

> Milestone M4 · Workstream: Testing · Scope: MVP

## Goal

The MVP is signed off: parity-matrix MVP rows are final on all three OSes and each ADR 0010 control has evidence accepted by security-auditor.

## Inputs

- `docs/parity-matrix.md`
- `docs/adr/0010-webview-trust-boundary.md`
- `docs/release/platform-verification.md`

## Outputs

- `docs/parity-matrix.md` — modify: MVP sign-off column
- `docs/release/mvp-signoff.md` — create

## Acceptance criteria

- [ ] mvp-signoff.md lists ADR 0010 controls 1–10 with evidence links and the auditor's verdict.
- [ ] Every MVP row is At parity, Degraded, Hidden or Replaced on each OS.

## Dependencies

- M3-20 — MVP feature gate review
- M4-10 — Release checklist
- M4-12 — Install and smoke-test on target platforms

## Owning agent

test-automator

## Reviewer agent

qa-expert, security-auditor, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
