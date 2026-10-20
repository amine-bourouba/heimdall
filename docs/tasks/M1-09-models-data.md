# M1-09 · Serde models for stats, history, todos, facets, debug log and file history

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Data-source models mirror their Swift counterparts, read the on-disk key spelling, emit camelCase over IPC, and pass the carried-over model tests.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Models/StatsCache.swift`
- `$SWIFT_REF/Poirot/Sources/Models/HistoryEntry.swift`
- `$SWIFT_REF/Poirot/Sources/Models/SessionTodo.swift`
- `$SWIFT_REF/Poirot/Sources/Models/SessionFacets.swift`
- `$SWIFT_REF/Poirot/Sources/Models/DebugLogEntry.swift`
- `$SWIFT_REF/Poirot/Sources/Models/FileVersion.swift`
- `$SWIFT_REF/PoirotTests/Models/StatsCacheTests.swift`
- `$SWIFT_REF/PoirotTests/Models/HistoryEntryTests.swift`
- `$SWIFT_REF/PoirotTests/Models/SessionTodoTests.swift`
- `$SWIFT_REF/PoirotTests/Models/SessionFacetsTests.swift`
- `$SWIFT_REF/PoirotTests/Models/DebugLogEntryTests.swift`
- `docs/spec/test-carryover.md`
- `docs/ipc-contract.md`

## Outputs

- `crates/heimdall-core/src/models/stats_cache.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/history_entry.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/session_todo.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/session_facets.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/debug_log_entry.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/file_version.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] `cargo test -p heimdall-core models::` passes with one Rust test per Swift test listed for these files.
- [ ] SessionFacets reads snake_case keys from disk (`underlying_goal`, …) and serialises camelCase over IPC.
- [ ] SessionTodo status accepts `in_progress`, `pending`, `completed`; unknown statuses do not fail the file.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
