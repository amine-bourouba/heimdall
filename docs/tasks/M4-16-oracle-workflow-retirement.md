# M4-16 · Retire the golden-oracle workflow

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

The golden-oracle workflow is deleted after the harness is retired.

## Inputs

- `docs/adr/0001-repo-layout-and-swift-reference.md`

## Outputs

- `.github/workflows/golden-oracle.yml` — delete

## Acceptance criteria

- [ ] No workflow references `tools/golden-oracle`.

## Dependencies

- M4-15 — Retire the Swift golden harness (conditional)

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
