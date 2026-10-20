# M1-14 · Project and session discovery with cwd-first project paths

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Project discovery reproduces SessionLoader (sessions-index fast path, JSONL fallback scan, sub-agent discovery) and resolves each project's path from the transcript `cwd` before any folder-name decoding.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/SessionLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Project.swift`
- `$SWIFT_REF/PoirotTests/Services/SessionLoaderTests.swift`
- `docs/adr/0003-platform-abstraction.md` — Project path resolution
- `docs/spec/claude-home-inventory.md`
- `fixtures/windows/MANIFEST.md` — format facts

## Outputs

- `crates/heimdall-core/src/projects/discovery.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/projects/sessions_index.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/projects/subagents.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/projects/project_path.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over SessionLoaderTests pass.
- [ ] The sessions-index fast path is used when present, and entries whose files are missing are skipped (SessionLoader.swift:104-121).
- [ ] Sub-agents are read from `<sessionId>/subagents/agent-*.jsonl`, excluding `agent-acompact-*`, with `.meta.json` agentType/description.
- [ ] Project path order: the `cwd` whose hyphen-encoding equals the folder name (case-insensitive, since Claude Code writes both `C:` and `c:`), else the most frequent `cwd`, then sessions-index `projectPath` or `originalPath` (both keys accepted), then `decode_project_dir_name`; `ProjectSummary.pathSource` reports which one won. A session that `cd`s into a subfolder must not change its project path (fixtures/windows MANIFEST facts 1-2).
- [ ] Orphaned transcripts (`<uuid>.orphaned-<ms>-<hex>.jsonl`) are skipped like any non-UUID stem, matching the Swift reference app; project folders that differ only by drive-letter case (`C--…` and `c--…`) stay separate projects.
- [ ] On fixtures/windows every project reports `pathSource = transcriptCwd` with a drive-letter path.
- [ ] `grep -rn "replace('-', \"/\")\|replace(\"-\", \"/\")" crates/heimdall-core/src` finds nothing.

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes
- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-13 — Transcript parser: header and summary tiers

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
