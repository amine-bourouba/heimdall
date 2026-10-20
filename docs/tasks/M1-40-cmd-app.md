# M1-40 · App commands: app info, sidebar counts, CLI detection

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

`get_app_info`, `get_sidebar_counts` and `detect_claude_cli` return contract-shaped data from core.

## Inputs

- `$SWIFT_REF/Poirot/Sources/App/AppState.swift:247-268`
- `$SWIFT_REF/Poirot/Sources/Views/Onboarding/OnboardingView.swift:281-297`
- `docs/ipc-contract.md`

## Outputs

- `src-tauri/src/commands/app.rs` — modify

## Acceptance criteria

- [ ] Sidebar counts match computeSidebarCounts for fixtures/synthetic.
- [ ] `detect_claude_cli` reports the found path (or none) and whether any project exists.

## Dependencies

- M1-15 — Prompt history reader
- M1-16 — Todo loader
- M1-24 — Loaders for commands, skills, plans, output styles and agents
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers
- M1-27 — Memory and models readers
- M1-31 — Cross-platform PATH search for claude and editor CLIs
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
