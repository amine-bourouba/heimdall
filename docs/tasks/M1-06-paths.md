# M1-06 · Claude home resolution, path normalisation and comparison keys

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

One `paths` module resolves Claude Code's home, normalises Windows and Unix path strings, and builds per-OS comparison keys, replacing every ad-hoc path build in Poirot.

## Inputs

- `$POIROT/Poirot/Sources/Services/SessionLoader.swift:61-67, 90-98, 314-353`
- `$POIROT/Poirot/Sources/Services/Config/SettingsWriter.swift:4-13`
- `$POIROT/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift:5, 113`
- `$POIROT/Poirot/Sources/Services/FileHistoryLoader.swift:170-178`
- `$POIROT/Poirot/Sources/Views/Todos/TodosOverviewView.swift:368-376`
- `docs/adr/0003-platform-abstraction.md` — Paths and case sensitivity

## Outputs

- `crates/heimdall-core/src/paths.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] `ClaudeHome` is built from `dirs::home_dir()` or from an explicit root passed by the caller (tests never touch the real home).
- [ ] The task verifies whether Claude Code honours `CLAUDE_CONFIG_DIR` (Claude Code docs or binary behaviour), records the evidence in the progress log, and implements it only if verified.
- [ ] Normalisation tests cover `C:\Users\a\x`, `c:/Users/a/x`, `\\?\C:\Users\a\x`, mixed separators, trailing separators and `/Users/a/x`.
- [ ] Comparison keys are case-insensitive for Windows and macOS and case-sensitive for Linux; every function takes a `TargetOs` argument so all variants run on any CI host.
- [ ] `same_file` reproduces FileHistoryLoader.sameFile semantics and also matches Windows paths (`src/a.rs` against `C:\proj\src\a.rs`).
- [ ] `is_unc()` detects `\\server\share` and `//server/share`.
- [ ] `decode_project_dir_name` ports SessionLoader.decodeProjectPath with an injected `exists` function and is documented as the last-resort fallback only.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
