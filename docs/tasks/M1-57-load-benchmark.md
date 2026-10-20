# M1-57 · Load-time benchmark against the performance budgets

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

A benchmark measures cold and warm project listing and the full parse of the largest transcript, and records results against the plan's budgets.

## Inputs

- `docs/port-plan.md` — §9 Performance budgets
- `fixtures/windows/`
- `fixtures/linux/`

## Outputs

- `crates/heimdall-core/examples/bench_load.rs` — create
- `docs/perf-baseline.md` — create

## Acceptance criteria

- [ ] `cargo run --release -p heimdall-core --example bench_load -- <claude_home>` prints timings, including for a generated tree of 30 projects × 500 sessions.
- [ ] docs/perf-baseline.md records results per OS and marks each budget pass or fail.

## Dependencies

- M0-14 — Capture a scrubbed real Windows ~/.claude tree
- M0-15 — Capture a scrubbed real Linux ~/.claude tree
- M1-12 — Transcript parser: full-parse tier, blocks, system content, tool input
- M1-13 — Transcript parser: header and summary tiers
- M1-14 — Project and session discovery with cwd-first project paths

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
