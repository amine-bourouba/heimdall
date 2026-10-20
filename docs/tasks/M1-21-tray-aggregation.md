# M1-21 · Tray snapshot aggregation

> Milestone M1 · Workstream: Rust data core · Scope: Later

## Goal

A pure function builds the tray snapshot (header stats plus up to 10 recent sessions, optionally fuzzy-filtered) exactly as MenuBarState does.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/MenuBar/MenuBarState.swift`
- `$SWIFT_REF/Poirot/Sources/Views/MenuBar/MenuBarView.swift`
- `$SWIFT_REF/PoirotTests/Views/MenuBarStateTests.swift`

## Outputs

- `crates/heimdall-core/src/data/tray.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over MenuBarStateTests pass.
- [ ] Recent sessions are sorted by startedAt descending, capped at 10, and filtered on session title or project name with `search::matcher`.

## Dependencies

- M1-14 — Project and session discovery with cwd-first project paths
- M1-20 — Stats cache reader, staleness rules and the full StatsComputer port
- M1-29 — Fuzzy and exact matcher with UTF-16 highlight ranges

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
