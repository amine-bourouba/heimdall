# M4-13 · Final README for the MVP release

> Milestone M4 · Workstream: Docs · Scope: MVP

## Goal

The README describes only features at parity (or explicitly degraded) in the MVP, supported platforms, install and attribution.

## Inputs

- `docs/parity-matrix.md`
- `docs/install.md`
- `e2e/visual/session.spec.ts-snapshots/`
- `e2e/visual/mvp-screens.spec.ts-snapshots/`

## Outputs

- `README.md` — modify

## Acceptance criteria

- [ ] Each listed feature maps to an MVP parity row At parity or Degraded; screenshots come from the Playwright baselines; the Poirot attribution remains.

## Dependencies

- M0-04 — Add LICENSE, NOTICE and a scoped README
- M2-24 — Visual baselines for the session screens
- M3-18 — Visual baselines for the remaining MVP screens
- M4-09 — Install guide for unsigned builds
- M4-12 — Install and smoke-test on target platforms

## Owning agent

technical-writer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
