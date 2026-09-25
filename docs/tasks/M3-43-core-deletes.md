# M3-43 · Core history, session and project deletes

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

History entries (one, or older than N days), sessions (file plus todos) and projects (folder) are deleted safely inside the Claude home.

## Inputs

- `$POIROT/Poirot/Sources/Views/History/HistoryListView.swift`
- `$POIROT/Poirot/Sources/App/AppState.swift:380-460`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `$POIROT/PoirotTests/Services/TodoLoaderTests.swift` — deleteTodos cases
- `docs/spec/test-carryover.md`

## Outputs

- `crates/heimdall-core/src/data/history_write.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/projects/delete.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] History rewrite uses the safe-write rules for JSONL and refuses if the file changed since it was read.
- [ ] Deletes refuse any path outside the Claude home (test).

## Dependencies

- M1-14 — Project and session discovery with cwd-first project paths
- M1-15 — Prompt history reader
- M1-16 — Todo loader
- M1-22 — Safe JSON write primitive for Claude Code's files
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

rust-engineer

## Reviewer agent

security-auditor, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
