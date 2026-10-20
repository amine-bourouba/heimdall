# M3-30 · Analytics and tray commands

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

`get_stats`, `get_tray_snapshot` and `open_in_main_window` return contract-shaped data and focus the main window on request.

## Inputs

- `docs/ipc-contract.md` — commands/analytics.rs, commands/tray.rs

## Outputs

- `src-tauri/src/commands/analytics.rs` — modify
- `src-tauri/src/commands/tray.rs` — modify

## Acceptance criteria

- [ ] `get_stats` reports whether data came from the cache or was recomputed.
- [ ] `open_in_main_window` shows and focuses the main window and emits `navigate`.

## Dependencies

- M1-20 — Stats cache reader, staleness rules and the full StatsComputer port
- M1-21 — Tray snapshot aggregation
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M1-38 — App state and the (mtime, size)-validated cache

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
