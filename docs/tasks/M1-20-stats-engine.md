# M1-20 · Stats cache reader, staleness rules and the full StatsComputer port

> Milestone M1 · Workstream: Rust data core · Scope: Later

## Goal

Analytics come from `stats-cache.json` only when it is not stale; otherwise a full port of StatsComputer recomputes them from transcripts, always with `costUSD: 0`.

## Inputs

- `$POIROT/Poirot/Sources/Services/StatsCacheLoader.swift`
- `$POIROT/Poirot/Sources/Services/StatsComputer.swift`
- `$POIROT/Poirot/Sources/Models/StatsCache.swift`
- `$POIROT/PoirotTests/Services/StatsComputerTests.swift`
- `$POIROT/PoirotTests/Models/StatsCacheTests.swift`
- `docs/spec/analytics.md`

## Outputs

- `crates/heimdall-core/src/data/stats_cache.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/data/stats.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over StatsComputerTests and the loader parts of StatsCacheTests pass.
- [ ] Staleness matches StatsCacheLoader.swift:36-51 (token series lags activity, or either series ends before the latest session day).
- [ ] Recomputation counts only main-thread (non-sidechain, UUID-named) sessions and fills per-day counts, per-model tokens, hour histogram, longest session and totals.

## Dependencies

- M0-10 — Behaviour spec for analytics, exports and the context ring
- M1-07 — Streaming JSONL reader with CRLF and truncation tolerance
- M1-09 — Serde models for stats, history, todos, facets, debug log and file history

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
