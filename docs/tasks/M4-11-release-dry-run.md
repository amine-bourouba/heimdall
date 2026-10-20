# M4-11 · Dry-run a tagged pre-release

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

A pre-release tag runs the release workflow end to end into a draft release, and updating from the previous build works on all three OSes.

## Inputs

- `docs/release/checklist.md`

## Outputs

- `docs/release/dry-run.md` — create

## Acceptance criteria

- [ ] dry-run.md records the workflow run, artifact list, checksums and per-OS update results.

## Dependencies

- M4-06 — Multi-platform release workflow
- M4-07 — Calendar-version bump workflow
- M4-10 — Release checklist

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
