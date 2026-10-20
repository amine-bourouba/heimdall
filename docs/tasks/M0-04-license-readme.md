# M0-04 · Add LICENSE, NOTICE and a scoped README

> Milestone M0 · Workstream: Docs · Scope: Infra

## Goal

The repository carries an MIT license that preserves the Swift reference app's copyright notice, a NOTICE naming the upstream source, and a README that claims no feature outside the MVP rows of the parity matrix.

## Inputs

- `$SWIFT_REF/LICENSE`
- `$SWIFT_REF/README.md` — do not copy its Usage Limits claims
- `docs/parity-matrix.md`
- `docs/adr/0011-app-identifier-and-release-repo.md`

## Outputs

- `LICENSE` — create
- `NOTICE` — create
- `README.md` — create

## Acceptance criteria

- [ ] LICENSE is MIT and contains `Copyright (c) 2026 Leonardo Cardoso` plus a Heimdall copyright line for the repository owner named in ADR 0011.
- [ ] NOTICE states Heimdall is derived from https://github.com/a7t-ai/poirot at commit a29dd6ee0e0133ef26bd067c164c66b160e13d86.
- [ ] `grep -n -i "usage limit" README.md` returns nothing.
- [ ] Every feature README.md names appears as an MVP row in docs/parity-matrix.md.

## Dependencies

- None

## Owning agent

technical-writer

## Reviewer agent

code-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
