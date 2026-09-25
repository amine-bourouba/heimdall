# M3-53 · Editor, terminal and reveal commands

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

`open_in_editor`, `open_terminal` and `reveal_in_file_manager` resolve ids to validated paths from cached data and run them through the platform module.

## Inputs

- `docs/ipc-contract.md` — commands/launch.rs
- `docs/adr/0010-webview-trust-boundary.md` — Process launch

## Outputs

- `src-tauri/src/commands/launch.rs` — modify: editor, terminal, reveal

## Acceptance criteria

- [ ] No command accepts a path from the webview.
- [ ] `open_terminal` copies `claude --resume <uuid>` and opens the preferred terminal at the project's working directory (TerminalLauncher.swift:4-16 behaviour).

## Dependencies

- M1-41 — Session read commands
- M1-44 — Config read commands
- M2-11 — open_external_url command
- M3-52 — Platform execution of launch actions

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
