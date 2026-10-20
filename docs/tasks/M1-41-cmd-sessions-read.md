# M1-41 · Session read commands

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

`list_projects`, `list_sessions`, `get_session`, `get_session_messages` and `list_recent_sessions` serve cached core data without blocking the event loop.

## Inputs

- `docs/ipc-contract.md` — commands/sessions.rs
- `docs/adr/0006-change-propagation-and-caching.md`

## Outputs

- `src-tauri/src/commands/sessions.rs` — modify

## Acceptance criteria

- [ ] Integration tests with the Tauri mock runtime on fixtures/screenshot cover each command.
- [ ] `get_session_messages` pages a cached full parse by index range; when the full parse is not cached yet, the first page is served from a partial streaming read so it renders before the whole file is parsed.
- [ ] Unknown project or session ids return `notFound`; ids failing validation return `invalidInput`.

## Dependencies

- M1-12 — Transcript parser: full-parse tier, blocks, system content, tool input
- M1-13 — Transcript parser: header and summary tiers
- M1-14 — Project and session discovery with cwd-first project paths
- M1-38 — App state and the (mtime, size)-validated cache

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
