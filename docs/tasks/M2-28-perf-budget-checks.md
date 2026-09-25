# M2-28 · Performance budget checks

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

Automated checks measure transcript scrolling at 10k+ blocks, Shiki lazy-load cost and per-route bundle sizes against the plan's budgets.

## Inputs

- `docs/port-plan.md` — §9 Performance budgets
- `docs/perf-baseline.md`

## Outputs

- `scripts/gen-large-transcript.mjs` — create
- `e2e/perf/transcript-scroll.spec.ts` — create
- `scripts/check-bundle-size.mjs` — create
- `docs/perf-baseline.md` — modify: append

## Acceptance criteria

- [ ] The scroll spec samples frame times with requestAnimationFrame on a generated 10k-block transcript and asserts the budget.
- [ ] check-bundle-size.mjs fails when any chunk exceeds its gzip ceiling.
- [ ] Measured values per OS are appended to docs/perf-baseline.md.

## Dependencies

- M1-57 — Load-time benchmark against the performance budgets
- M2-20 — Virtualised conversation view
- M2-23 — Playwright and real-app E2E harness

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
