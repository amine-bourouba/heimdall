# M4-10 · Release checklist

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

A checklist proves, for each release, that updater signatures verify and checksums match on every OS.

## Inputs

- `.github/workflows/release.yml`
- `docs/release/updater-key.md`

## Outputs

- `docs/release/checklist.md` — create

## Acceptance criteria

- [ ] Steps: CI green and parity gate passed; tag; workflow; checksum match on each OS; update from the previous version succeeds on each OS (the signature check is part of that); install and uninstall; prefs survive the update; rollback plan.

## Dependencies

- M4-06 — Multi-platform release workflow

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
