# M1-52 · Commit the golden outputs

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

The oracle's JSON outputs are committed under `fixtures/golden/` with their provenance.

## Inputs

- `.github/workflows/golden-oracle.yml`
- `fixtures/README.md`

## Outputs

- `fixtures/golden/**` — create
- `fixtures/golden/README.md` — create

## Acceptance criteria

- [ ] README records the upstream SHA, workflow run URL and date; one folder per fixture set.

## Dependencies

- M1-51 — macOS workflow that runs the golden oracle

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
