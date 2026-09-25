# M4-07 · Calendar-version bump workflow

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

A manually dispatched workflow bumps the calendar version (YY.M.BUILD, semver-valid) in package.json, commits and tags it.

## Inputs

- `$POIROT/.github/workflows/version-bump.yml`
- `docs/port-plan.md` — decision P-10

## Outputs

- `.github/workflows/version-bump.yml` — create

## Acceptance criteria

- [ ] Versions have no zero padding (26.8.5, not 26.08.5) so Tauri, Cargo and npm accept them.
- [ ] BUILD resets when the month changes; a forced-version input is validated.
- [ ] Only `package.json` and `package-lock.json` change; tauri.conf.json reads the version from package.json.

## Dependencies

- M1-02 — Create package.json with the full frontend toolchain
- M1-03 — Scaffold the Tauri app crate with final identity, CSP and module stubs

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
