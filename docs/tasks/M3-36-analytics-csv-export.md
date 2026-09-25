# M3-36 · Analytics CSV export and PNG save command

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

The four analytics CSV exports are generated in Rust and saved through a Rust-opened dialog, and `save_png` writes validated PNG bytes the same way.

## Inputs

- `$POIROT/Poirot/Sources/Views/Analytics/Export/AnalyticsCSVExporter.swift`
- `$POIROT/PoirotTests/Views/AnalyticsCSVExporterTests.swift`
- `docs/ipc-contract.md` — commands/export.rs

## Outputs

- `crates/heimdall-core/src/export/analytics_csv.rs` — fill the placeholder created by M1-05
- `src-tauri/src/commands/export.rs` — modify: analytics CSV and save_png

## Acceptance criteria

- [ ] Carried-over AnalyticsCSVExporterTests pass in Rust.
- [ ] `save_png` rejects bytes without the PNG signature.

## Dependencies

- M1-20 — Stats cache reader, staleness rules and the full StatsComputer port
- M3-09 — Session export commands

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
