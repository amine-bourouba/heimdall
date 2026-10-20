# M3-38 · Add the PNG export dependency

> Milestone M3 · Workstream: Release and packaging · Scope: Later

## Goal

The dependency chosen in the PNG spike is added and audited.

## Inputs

- `docs/spikes/png-export.md`

## Outputs

- `package.json` — modify
- `package-lock.json` — modify

## Acceptance criteria

- [ ] `npm audit --audit-level=high` passes; the licence is MIT-compatible.

## Dependencies

- M3-37 — Spike: PNG export approach (open risk R-7)
- M4-08 — Per-OS developer setup scripts

## Owning agent

dependency-manager

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
