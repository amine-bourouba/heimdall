# M0-16 · Define and run the upstream Poirot sync check

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

A written, repeatable procedure lists upstream Poirot changes since the pinned commit and turns genuinely new capabilities into new parity-matrix rows.

## Inputs

- `docs/adr/0001-repo-layout-and-swift-reference.md`
- `docs/parity-matrix.md`
- `$POIROT/` — git history

## Outputs

- `docs/spec/upstream-sync.md` — create

## Acceptance criteria

- [ ] The document gives the exact commands (`git -C $POIROT fetch origin` and `git -C $POIROT log --oneline a29dd6e..origin/main -- Poirot/Sources README.md SHOWCASE.md`) and a monthly cadence.
- [ ] It holds a first-run log (date, upstream head SHA, commits found, proposed matrix rows or 'none').
- [ ] It states that new capabilities become new matrix rows and never reopen a finished milestone.

## Dependencies

- None

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
